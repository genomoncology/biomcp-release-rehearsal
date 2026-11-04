use crate::{Ctx, Outcome, fan_out, status};
use biodata::Text;
use biomcp_adapters::{europepmc, litvar, pubmed};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::ReplyBody;
use biomcp_types::{
    ArticleItem, ArticleKey, ArticleSearch, ArticleSource, Source, SourceOutcome, SourceState,
};
use std::time::Duration;
pub(super) struct ResultLeg {
    pub(super) dropped: bool,
    pub(super) source: ArticleSource,
    pub(super) outcome: SourceOutcome,
    pub(super) items: Vec<ArticleItem>,
    pub(super) problems: Vec<biomcp_types::Problem>,
    pub(super) dois: Vec<Option<biodata::Doi>>,
    pub(super) elapsed: Duration,
    pub(super) error: Option<RequestError>,
}
pub(super) enum SearchResult {
    Answer,
    Unavailable,
    Items(Vec<ArticleItem>),
}
enum Preparation<'a> {
    Ready(Option<PreparedLitvar<'a>>),
    Answer,
}
pub(super) fn search<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    resolved: &super::term::Resolved,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<SearchResult, RequestError> {
    let mut prepared = match prepare_litvar(cx, query, resolved.symbol.as_deref(), deadline, out) {
        Preparation::Ready(prepared) => prepared,
        Preparation::Answer => return Ok(SearchResult::Answer),
    };
    let start = *deadline;
    let jobs: Vec<Box<dyn FnOnce() -> ResultLeg + Send + '_>> = query
        .sources()
        .iter()
        .filter_map(|selector| {
            let job: Box<dyn FnOnce() -> ResultLeg + Send + '_> = match selector {
                ArticleSource::Pubmed => {
                    Box::new(move || pubmed_search(cx, query, &resolved.names, start))
                }
                ArticleSource::Europepmc => Box::new(move || europe_search(cx, query, start)),
                ArticleSource::Litsense => {
                    Box::new(move || super::named::litsense(cx, query, start))
                }
                ArticleSource::Pubtator => {
                    Box::new(move || super::named::pubtator(cx, query, resolved.entrez, start))
                }
                ArticleSource::SemanticScholar => {
                    Box::new(move || super::semantic_scholar::search(cx, query, start))
                }
                ArticleSource::Litvar => {
                    let prepared = prepared.take()?;
                    Box::new(move || litvar_search(prepared))
                }
            };
            Some(job)
        })
        .collect();
    let mut results = fan_out::run(jobs);
    let elapsed = results.iter().map(|r| r.elapsed).max().unwrap_or_default();
    *deadline = deadline.after_stage(elapsed);
    resolve_rows(cx, query, &mut results, deadline, out)?;
    let answered = results
        .iter()
        .any(|r| r.error.is_none() || !r.items.is_empty());
    let error = results
        .iter()
        .filter_map(|r| r.error)
        .find(|e| status::local(*e).is_some());
    for result in &results {
        out.dropped |= result.dropped;
        out.sources.push(result.outcome.clone());
        status::append(&mut out.problems, result.problems.clone());
    }
    if let Some(error) = error {
        return Err(error);
    }
    if !answered {
        crate::status::item(out, false, true, false);
        return Ok(SearchResult::Unavailable);
    }
    Ok(SearchResult::Items(super::fold::fold(
        results.into_iter().map(|r| (r.source, r.items)).collect(),
        query.sources().len() > 1,
    )))
}
fn resolve_rows<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    results: &mut [ResultLeg],
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let mut join_failures = Vec::new();
    for result in results.iter_mut() {
        if result.source == ArticleSource::SemanticScholar
            && result.items.iter().enumerate().any(|(index, item)| {
                item.pmid.is_none()
                    && (result.dois.get(index).is_some_and(Option::is_some)
                        || super::semantic_scholar::doi(item).is_some())
            })
        {
            let resolved = super::semantic_scholar::resolve_dois_with(
                cx,
                &mut result.items,
                deadline,
                &result.dois,
            );
            status::append(&mut out.problems, resolved.problems);
            if let Some(error) = resolved.error {
                if status::local(error).is_some() {
                    return Err(error);
                }
                join_failures.push(resolved.source);
            } else if resolved.source.state != SourceState::Ok {
                join_failures.push(resolved.source);
            } else if !results_have_pubmed(query) {
                out.sources.push(resolved.source);
            }
        }
    }
    for failure in join_failures {
        if let Some(existing) = results
            .iter_mut()
            .find(|r| r.outcome.source == failure.source)
        {
            status::retain_source_state(&mut existing.outcome, &failure);
        } else {
            out.sources.push(failure);
        }
    }
    Ok(())
}
fn pubmed_search<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    names: &[String],
    deadline: Deadline<'a>,
) -> ResultLeg {
    let leg = cx.fetch.leg(&pubmed::SPEC, deadline).fresh(cx.fresh);
    let start = leg.remaining();
    let (result, term) = pubmed::search_query(&leg, query, names, cx.ncbi_email.as_ref());
    let elapsed = start.saturating_sub(leg.remaining());
    let error = result.as_ref().err().copied();
    let mut outcome = status::source(Source::PubMed, leg.finish(), error);
    outcome.term = Text::new(&term).ok();
    let dropped = false;
    let mut problems = Vec::new();
    let items = match result {
        Ok(ids) => {
            outcome.count = Some(ids.count);
            outcome.retrieved_count = Some(ids.ids.len() as u32);
            outcome.notices.extend(ids.notices);
            problems.extend(ids.problems);
            ids.ids
                .into_iter()
                .map(|id| ArticleItem {
                    key: Some(ArticleKey::Pubmed(id)),
                    pmid: Some(id),
                    header: None,
                    cited_by: None,
                    intents: None,
                    influential: None,
                    contexts: None,
                    found_by: Default::default(),
                    sentence: None,
                })
                .collect()
        }
        Err(_) => Vec::new(),
    };
    ResultLeg {
        dropped,
        problems,
        source: ArticleSource::Pubmed,
        outcome,
        items,
        dois: Vec::new(),
        elapsed,
        error,
    }
}
fn europe_search<'a>(cx: &Ctx<'a>, query: &ArticleSearch, deadline: Deadline<'a>) -> ResultLeg {
    let leg = cx.fetch.leg(&europepmc::SPEC, deadline).fresh(cx.fresh);
    let start = leg.remaining();
    let (result, term) = europepmc::search_query(&leg, query);
    let elapsed = start.saturating_sub(leg.remaining());
    let error = result.as_ref().err().copied();
    let mut outcome = status::source(Source::EuropePmc, leg.finish(), error);
    outcome.term = Text::new(&term).ok();
    let mut dropped = false;
    let mut problems = Vec::new();
    let items = match result {
        Ok(reply) => {
            dropped |= reply.refusals == biomcp_adapters::Refusal::Refused;
            outcome.count = Some(reply.count);
            outcome.retrieved_count = Some(reply.items.len() as u32);
            problems = reply.problems;
            reply.items
        }
        Err(_) => Vec::new(),
    };
    ResultLeg {
        dropped,
        problems,
        source: ArticleSource::Europepmc,
        outcome,
        items,
        dois: Vec::new(),
        elapsed,
        error,
    }
}
pub(super) fn failed(sources: &[SourceOutcome]) -> bool {
    sources.iter().any(|s| s.state == SourceState::Failed)
}

type PreparedLitvar<'a> = (biomcp_fetch::Leg<'a>, Result<litvar::Variant, RequestError>);
fn prepare_litvar<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    gene: Option<&str>,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Preparation<'a> {
    if !query.sources().contains(&ArticleSource::Litvar) {
        return Preparation::Ready(None);
    }
    let leg = cx.fetch.leg(&litvar::SPEC, *deadline).fresh(cx.fresh);
    let start = leg.remaining();
    let result =
        litvar::select(&leg, gene, query.variant().map_or("", Text::as_str)).map(|mapped| {
            out.include_refusals(mapped.refusals);
            out.problems.extend(mapped.problems);
            mapped.value
        });
    *deadline = super::consume(*deadline, start, leg.remaining());
    match result {
        Ok(litvar::Selection::Exact(selected)) => Preparation::Ready(Some((leg, Ok(selected)))),
        Err(error) => Preparation::Ready(Some((leg, Err(error)))),
        Ok(litvar::Selection::Candidates(candidates)) => {
            out.sources
                .push(status::source(Source::LitVar, leg.finish(), None));
            status::search_for(out, &[Source::LitVar]);
            if candidates.is_empty() {
                super::empty_litvar::recover(query, out);
            }
            for candidate in &candidates {
                for variant in candidate.rsid.iter().chain(candidate.name.iter()) {
                    if query.variant() == Some(variant) {
                        continue;
                    }
                    out.next.push(crate::next::Step::run(
                        biomcp_types::StepKind::Disambiguate,
                        &super::next::search_sources(query),
                        biomcp_types::Request::SearchArticle(query.with_variant(variant.clone())),
                        cx.fresh,
                    ));
                }
            }
            out.body = (biomcp_types::ArticlePage {
                items: Box::default(),
                candidates,
            })
            .with_query(query.clone());
            Preparation::Answer
        }
    }
}
fn litvar_search((leg, selected): PreparedLitvar<'_>) -> ResultLeg {
    let start = leg.remaining();
    let term = selected
        .as_ref()
        .ok()
        .map(|selected| selected.term().clone());
    let result = selected.and_then(|selected| litvar::publications(&leg, selected));
    let elapsed = start.saturating_sub(leg.remaining());
    let error = result.as_ref().err().copied();
    let mut outcome = status::source(Source::LitVar, leg.finish(), error);
    outcome.term = term;
    let mut dropped = false;
    let mut problems = Vec::new();
    let items = match result {
        Ok(litvar::Search {
            term,
            count,
            ids,
            problems: rejected,
            refusals,
        }) => {
            dropped |= refusals == biomcp_adapters::Refusal::Refused;
            problems = rejected;
            outcome.term = Some(term);
            outcome.count = count;
            outcome.retrieved_count = Some(ids.len() as u32);
            ids.into_iter()
                .map(|id| ArticleItem {
                    key: Some(ArticleKey::Pubmed(id)),
                    pmid: Some(id),
                    header: None,
                    cited_by: None,
                    intents: None,
                    influential: None,
                    contexts: None,
                    found_by: Default::default(),
                    sentence: None,
                })
                .collect()
        }
        Err(_) => Vec::new(),
    };
    ResultLeg {
        dropped,
        problems,
        source: ArticleSource::Litvar,
        outcome,
        items,
        dois: Vec::new(),
        elapsed,
        error,
    }
}

fn results_have_pubmed(query: &ArticleSearch) -> bool {
    query.sources().contains(&ArticleSource::Pubmed)
}
