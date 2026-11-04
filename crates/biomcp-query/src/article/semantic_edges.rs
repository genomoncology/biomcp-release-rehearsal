//! Semantic Scholar edge pools retain source order and per-edge evidence.
use crate::{Action, Ctx, Outcome, status};

use biomcp_adapters::semantic_scholar;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{ArticleEdges, ArticleItem, Reason, Source, StepKind};
pub(super) fn create<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEdges,
    references: bool,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Option<Vec<ArticleItem>>, RequestError> {
    let Some(input) = resolve_input(cx, query, references, deadline, out)? else {
        return Ok(None);
    };
    let leg = cx
        .fetch
        .leg(&semantic_scholar::SPEC, *deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let result = collect(
        &leg,
        &input,
        references,
        &mut out.problems,
        &mut out.dropped,
    );
    *deadline = super::consume(*deadline, started, leg.remaining());
    let error = result.as_ref().err().copied();
    let mut source = status::source(Source::SemanticScholar, leg.finish(), error);
    let Window {
        mut items,
        count,
        truncated,
        dois,
    } = match result {
        Ok(value) => value,
        Err(error) => {
            out.sources.push(source);
            crate::status::item(out, false, true, false);
            failure(query, references, out);
            return if status::local(error).is_some() {
                Err(error)
            } else {
                Ok(None)
            };
        }
    };
    source.count = (!truncated).then_some(count);
    source.retrieved_count = Some(count as u32);
    if truncated {
        source
            .notices
            .push(biomcp_types::SourceNotice::GraphTruncated {
                fallback: if references {
                    "article references <PMID> --source icite"
                } else {
                    "article citations <PMID> --source icite"
                }
                .parse()
                .map_err(|_| biomcp_fetch::CacheError::Internal)?,
            });
    }
    out.sources.push(source);
    if items
        .iter()
        .zip(&dois)
        .any(|(item, doi)| item.pmid.is_none() && doi.is_some())
    {
        let joined = super::semantic_scholar::resolve_dois_with(cx, &mut items, deadline, &dois);
        out.sources.push(joined.source);
        status::append(&mut out.problems, joined.problems);
        if let Some(error) = joined.error.and_then(status::local) {
            return Err(error);
        }
    }
    for item in &mut items {
        if item.pmid.is_some() && item.header.as_ref().is_some_and(|h| h.types_unknown) {
            item.header = None;
        }
    }
    let mut seen = std::collections::HashSet::new();
    items.retain(|item| {
        item.key.is_none() || seen.insert(serde_json::to_string(&item.key).unwrap_or_default())
    });
    Ok(Some(items))
}
struct Window {
    items: Vec<ArticleItem>,
    count: u64,
    truncated: bool,
    dois: Vec<Option<biodata::Doi>>,
}
fn collect(
    leg: &biomcp_fetch::Leg<'_>,
    input: &biomcp_types::ArticleInput,
    references: bool,
    problems: &mut Vec<biomcp_types::Problem>,
    dropped: &mut bool,
) -> Result<Window, RequestError> {
    let mut offset = None;
    let mut items = Vec::new();
    let mut dois = Vec::new();
    let mut count = 0;
    for _ in 0..5 {
        let reply = semantic_scholar::edges(leg, input, references, offset)?;
        *dropped |= reply.refusals == biomcp_adapters::Refusal::Refused;
        problems.extend(reply.problems);
        count += reply.data.len() as u64;
        for edge in reply.data {
            let paper = if references {
                &edge.cited_paper
            } else {
                &edge.citing_paper
            };
            dois.push(
                paper
                    .as_ref()
                    .and_then(|paper| paper.external_ids.as_ref())
                    .and_then(|ids| ids.doi.clone()),
            );
            let item = edge
                .item(references, problems)
                .map_err(|_| biomcp_fetch::bad_reply(Source::SemanticScholar, None))?;
            items.push(item);
        }
        match reply.next {
            None => {
                return Ok(Window {
                    items,
                    count,
                    truncated: false,
                    dois,
                });
            }
            Some(next) if next > offset.unwrap_or(0) => offset = Some(next),
            Some(_) => return Err(biomcp_fetch::bad_reply(Source::SemanticScholar, None)),
        }
    }
    Ok(Window {
        items,
        count,
        truncated: true,
        dois,
    })
}

fn failure(query: &ArticleEdges, references: bool, out: &mut Outcome) {
    if out
        .sources
        .iter()
        .any(|s| s.source == Source::SemanticScholar && s.reason == Some(Reason::RateLimited))
    {
        out.next.push(crate::next::Step::new(
            StepKind::Key,
            &[biomcp_types::Source::SemanticScholar],
            Action::KeyHelp(biomcp_fetch::SecretName::SemanticScholarApiKey),
        ));
    }
    out.next.push(crate::next::Step::run(
        StepKind::Fresh,
        &[biomcp_types::Source::SemanticScholar],
        if references {
            biomcp_types::Request::ArticleReferences(query.page(0, None))
        } else {
            biomcp_types::Request::ArticleCitations(query.page(0, None))
        },
        true,
    ));
}

fn resolve_input<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEdges,
    references: bool,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Option<biomcp_types::ArticleInput>, RequestError> {
    if !matches!(query.input(), biomcp_types::ArticleInput::Pmcid(_)) {
        return Ok(Some(query.input().clone()));
    }
    let leg = cx
        .fetch
        .leg(&biomcp_adapters::pubmed::SPEC, *deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let result = biomcp_adapters::pubmed::resolve_ids(&leg, query.input(), cx.ncbi_email.as_ref());
    *deadline = super::consume(*deadline, started, leg.remaining());
    let error = result.as_ref().err().copied();
    let mut source = status::source(Source::PubMed, leg.finish(), error);
    let input = match result {
        Ok(ids) if ids.len() == 1 => ids.first().copied().map(biomcp_types::ArticleInput::Pmid),
        Ok(ids) => {
            source.state = biomcp_types::SourceState::Failed;
            source.reason = Some(if ids.is_empty() {
                Reason::NotFound
            } else {
                Reason::BadReply
            });
            None
        }
        Err(error) if status::local(error).is_some() => return Err(error),
        Err(_) => None,
    };
    out.sources.push(source);
    if input.is_none() {
        crate::status::item(out, false, true, false);
        failure(query, references, out);
    }
    Ok(input)
}
