//! PubTator3 abstract annotation export, joined by each document's own PMID.
use biomcp_fetch::{Input, Read};
mod map;
mod search;
pub use search::{Search, query_term, search};
mod raw;
use crate::{Mapped, quotas};
use biodata::Pmid;
use biomcp_fetch::{Arg, Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::{Annotations, Section, Source};
use std::time::Duration;
/// Conservative application batch bound; DOI expansions are batched by the caller.
pub const BATCH_LIMIT: usize = 20;
/// Public export policy; its quota is local policy, not a claimed provider limit.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "NCBI policies; publisher rights in input text",
        url: "https://www.ncbi.nlm.nih.gov/home/about/policies/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::PubTator,
    base: "https://www.ncbi.nlm.nih.gov/research/pubtator3-api",
    key: KeyUse::None,
    quota: &quotas::PUBTATOR,
    deadline: Duration::from_secs(8),
};
const EXPORT: Endpoint = SPEC
    .get("/publications/export/biocjson")
    .fresh(Freshness::Days(7));
/// One PMID's independently mapped export section.
#[derive(Debug)]
pub struct ArticleAnnotations {
    /// Requested document identity.
    pub id: Pmid,
    /// Mapped section and optional field problems.
    pub annotations: Mapped<Section<Annotations>>,
}
/// Export unique PMIDs in one bounded request and retain absent documents as empty sections.
pub fn annotations(leg: &Leg<'_>, ids: &[Pmid]) -> Result<Vec<ArticleAnnotations>, RequestError> {
    if ids.is_empty() || ids.len() > BATCH_LIMIT {
        return Err(biomcp_fetch::SourceError {
            source: Source::PubTator,
            reason: biomcp_types::Reason::BadReply,
            status: None,
            tls_cause: None,
            retry_after: None,
        }
        .into());
    }
    let names: Vec<_> = ids
        .iter()
        .map(|id| id.to_string().into_boxed_str())
        .collect();
    let reply = leg
        .call(
            &EXPORT,
            Input::Params(&[("pmids", Arg::Ids(&names))]),
            Read::<raw::Export>::json()
                .filter_body(&|body| crate::article_rights::pubtator::filter(leg, body)),
        )?
        .value;
    let malformed_identity = reply.documents.iter().any(|v| v.as_ref().is_none());
    let mut results: Vec<_> = ids
        .iter()
        .map(|id| {
            let document = reply
                .documents
                .iter()
                .filter_map(crate::raw::Field::as_ref)
                .find(|doc| doc.id == *id);
            ArticleAnnotations {
                id: *id,
                annotations: map::document(*id, document),
            }
        })
        .collect();
    if malformed_identity && let Some(first) = results.first_mut() {
        first.annotations.refusals = crate::Refusal::Refused;
        crate::mapped::push_problem(
            &mut first.annotations.problems,
            Source::PubTator,
            None,
            "pmid",
        );
    }
    Ok(results)
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
