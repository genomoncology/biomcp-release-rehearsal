//! Locate the citing paper without treating an unresolved DOI as a missing PMC copy.
use super::{Snapshot, source};
use crate::{Ctx, Outcome, status};
use biodata::{Pmcid, Pmid};

use biomcp_adapters::pubmed;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::ArticleInput;
use biomcp_types::{Reason, Source};

pub(super) fn resolve<'a>(
    cx: &Ctx<'a>,
    input: &ArticleInput,
    snapshot: &mut Snapshot,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Option<(Pmcid, Option<Pmid>)>, RequestError> {
    let ArticleInput::Doi(doi) = input else {
        let pmid = if let ArticleInput::Pmid(id) = input {
            Some(*id)
        } else {
            None
        };
        return crate::article::text::resolve(cx, input, deadline, out)
            .map(|id| id.map(|id| (id, pmid)));
    };
    let leg = cx.fetch.leg(&pubmed::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let result = (|| {
        let ids = pubmed::resolve_ids(&leg, input, cx.ncbi_email.as_ref())?;
        if ids.is_empty() {
            return Err(no_join());
        }
        let mapped = pubmed::summaries(&leg, &ids, cx.ncbi_email.as_ref())?;
        out.include_refusals(mapped.refusals);
        status::append(&mut out.problems, mapped.problems);
        let mut exact = mapped.value.into_iter().filter(|header| {
            header.doi.as_ref() == Some(doi) && header.pmid.is_some_and(|id| ids.contains(&id))
        });
        let header = exact.next().ok_or_else(no_join)?;
        if exact.next().is_some() {
            return Err(no_join());
        }
        Ok(header.pmcid.map(|id| (id, header.pmid)))
    })();
    *deadline = crate::article::consume(*deadline, started, leg.remaining());
    source(
        snapshot,
        status::source(Source::PubMed, leg.finish(), result.as_ref().err().copied()),
    );
    result
}
fn no_join() -> RequestError {
    SourceError {
        source: Source::PubMed,
        reason: Reason::NoJoinId,
        status: None,
        tls_cause: None,
        retry_after: None,
    }
    .into()
}
