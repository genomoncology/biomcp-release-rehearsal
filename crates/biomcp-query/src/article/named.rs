use super::sources::ResultLeg;
use crate::{Ctx, status};
use biodata::{NcbiGeneId, Text};

use biomcp_adapters::{litsense, pubtator};
use biomcp_fetch::Deadline;
use biomcp_types::{ArticleItem, ArticleKey, ArticleSearch, ArticleSource, Reason, Source};
/// Literal gene input and words remain separate from HGNC's resolved entity join.
pub(super) fn litsense<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    deadline: Deadline<'a>,
) -> ResultLeg {
    let leg = cx.fetch.leg(&litsense::SPEC, deadline).fresh(cx.fresh);
    let start = leg.remaining();
    let (result, term) = litsense::search_query(&leg, query);
    let elapsed = start.saturating_sub(leg.remaining());
    let error = result.as_ref().err().copied();
    let mut outcome = status::source(Source::LitSense, leg.finish(), error);
    outcome.term = Text::new(&term).ok();
    let dropped = result
        .as_ref()
        .is_ok_and(|r| r.refusals == biomcp_adapters::Refusal::Refused);
    let (items, problems) = result.map(|r| (r.value, r.problems)).unwrap_or_default();
    if error.is_none() {
        outcome.count = Some(items.len() as u64);
        outcome.retrieved_count = Some(items.len() as u32);
    }
    ResultLeg {
        dropped,
        problems,
        source: ArticleSource::Litsense,
        outcome,
        items,
        dois: Vec::new(),
        elapsed,
        error,
    }
}
pub(super) fn pubtator<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    entrez: Option<NcbiGeneId>,
    deadline: Deadline<'a>,
) -> ResultLeg {
    let leg = cx.fetch.leg(&pubtator::SPEC, deadline).fresh(cx.fresh);
    let start = leg.remaining();
    let term = pubtator::query_term(query, entrez);
    let result =
        if query.gene().is_some() && entrez.is_none() && !cx.fetch.disabled(Source::PubTator) {
            Err(biomcp_fetch::SourceError {
                source: Source::PubTator,
                reason: Reason::NoJoinId,
                status: None,
                tls_cause: None,
                retry_after: None,
            }
            .into())
        } else {
            pubtator::search(&leg, &term)
        };
    let elapsed = start.saturating_sub(leg.remaining());
    let error = result
        .as_ref()
        .err()
        .copied()
        .or_else(|| result.as_ref().ok().and_then(|r| r.error));
    let mut outcome = status::source(Source::PubTator, leg.finish(), error);
    outcome.term = Text::new(&term).ok();
    let items = match result {
        Ok(reply) => {
            outcome.count = Some(reply.count);
            outcome.retrieved_count = Some(reply.ids.len() as u32);
            reply
                .ids
                .into_iter()
                .map(|id| ArticleItem {
                    text_withheld: None,
                    context_license: None,
                    key: Some(ArticleKey::Pubmed(id)),
                    pmid: Some(id),
                    found_by: Default::default(),
                    cited_by: None,
                    intents: None,
                    influential: None,
                    contexts: None,
                    header: None,
                    sentence: None,
                })
                .collect()
        }
        Err(_) => Vec::new(),
    };
    ResultLeg {
        dropped: false,
        problems: Vec::new(),
        source: ArticleSource::Pubtator,
        outcome,
        items,
        dois: Vec::new(),
        elapsed,
        error,
    }
}
