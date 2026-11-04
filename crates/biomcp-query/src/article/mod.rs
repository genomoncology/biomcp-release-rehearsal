//! PubMed snapshots and stable pages through one checked command consumer.
use biomcp_types::ReplyBody;
mod edges;
mod empty_litvar;
pub(crate) mod evidence;
mod fold;
pub(crate) mod get;
mod named;
pub(crate) mod next;
mod order;
mod page;
pub mod pdf;
mod preprint;
mod protein_change;
pub(crate) mod recommend;
pub(crate) mod save;
mod sections;
mod semantic_edges;
pub(crate) mod semantic_scholar;
mod sources;
pub mod supplement;
mod term;
pub mod text;
use crate::{Action, Ctx, Outcome, status, store::Store};
use biodata::Text;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::{ArticlePage, ArticleSearch, ReplyData, Request, StepKind};
use sha2::{Digest, Sha256};
/// Run one article command under a single absolute deadline.
pub(crate) fn search(cx: &Ctx<'_>, query: &ArticleSearch) -> Outcome {
    let mut out = empty(
        (ArticlePage {
            items: Box::default(),
            candidates: Vec::new(),
        })
        .with_query(query.clone()),
    );
    if let Some((source, option)) = unsupported(query) {
        out.body = ReplyData::Usage(biomcp_types::UsageError::UnsupportedFilter { source, option });
        return out;
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    let result = execute(cx, query, &mut deadline, &mut out);
    command.finish(deadline.remaining());
    if let Err(error) = result {
        out.local = status::local(error);
        crate::status::item(&mut out, false, true, false);
    }
    if out.sources.iter().any(|source| {
        source.source == biomcp_types::Source::SemanticScholar
            && source.reason == Some(biomcp_types::Reason::RateLimited)
    }) {
        out.next.insert(
            0,
            crate::next::Step::new(
                StepKind::Key,
                &[biomcp_types::Source::SemanticScholar],
                Action::KeyHelp(biomcp_fetch::SecretName::SemanticScholarApiKey),
            ),
        );
    }
    out
}
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let mut store = Store::open(cx.fetch.cache_access()?, *deadline)?;
    let bytes = serde_json::to_vec(&query.identity()).map_err(|_| CacheError::Encoding)?;
    let key = Sha256::digest([b"biomcp.article_page/1\0".as_slice(), &bytes].concat());
    let explicit = query.pool().map(Text::as_str);
    let stored = if cx.fresh && explicit.is_none() {
        None
    } else {
        store.find(&key, explicit)?
    };
    let pool = match stored {
        Some(pool) if pool.key != key.as_slice() => {
            pool_error(query, true, out);
            return Ok(());
        }
        Some(pool) => pool,
        None if explicit.is_some() => {
            pool_error(query, false, out);
            return Ok(());
        }
        None => {
            let resolved = if query
                .sources()
                .contains(&biomcp_types::ArticleSource::Pubmed)
                || query
                    .sources()
                    .contains(&biomcp_types::ArticleSource::Litvar)
                || query
                    .sources()
                    .contains(&biomcp_types::ArticleSource::Pubtator)
                    && !cx.fetch.disabled(biomcp_types::Source::PubTator)
            {
                let Some(term) = term::resolve(cx, query, deadline, out)? else {
                    return Ok(());
                };
                term
            } else {
                term::Resolved {
                    names: Vec::new(),
                    symbol: None,
                    entrez: None,
                }
            };
            let mut items = match sources::search(cx, query, &resolved, deadline, out)? {
                sources::SearchResult::Items(items) => items,
                sources::SearchResult::Answer => return Ok(()),
                sources::SearchResult::Unavailable => {
                    retry(query, None, out);
                    return Ok(());
                }
            };
            if query.sort() == biomcp_types::ArticleSort::Citations
                && !order::apply(cx, deadline, &mut items, out)?
            {
                retry(query, None, out);
                return Ok(());
            }
            store.set_deadline(*deadline);
            match crate::publish::keep(
                store
                    .publish(&key, &items, &out.sources, &out.problems, out.dropped)
                    .map_err(Into::into),
                out,
            )? {
                Some(pool) => pool,
                None => {
                    page::fetched(page::Cursor::Search(query), items, out);
                    return Ok(());
                }
            }
        }
    };
    out.sources = pool.sources.clone();
    let result = page::read(cx, page::Cursor::Search(query), &mut store, &pool, out);
    *deadline = store.deadline();
    result
}
fn pool_error(query: &ArticleSearch, mismatch: bool, out: &mut Outcome) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &next::search_sources(query),
        Request::SearchArticle(query.page(0, None)),
        false,
    ));
}
fn retry(query: &ArticleSearch, pool: Option<Text>, out: &mut Outcome) {
    out.next.push(crate::next::Step::run(
        StepKind::Fresh,
        &next::search_sources(query),
        Request::SearchArticle(query.page(query.offset(), pool)),
        true,
    ));
}
fn empty(body: ReplyData) -> Outcome {
    crate::status::ready(body)
}

// Fetch legs deliberately own independent replay clocks. A sequential article
// command debits each leg before starting another; real deadlines remain absolute.
pub(super) fn consume<'a>(
    deadline: Deadline<'a>,
    started: std::time::Duration,
    remaining: std::time::Duration,
) -> Deadline<'a> {
    deadline.after_stage(started.saturating_sub(remaining))
}

/// Page one citation graph direction.
pub(crate) fn edges(cx: &Ctx<'_>, query: &biomcp_types::ArticleEdges, references: bool) -> Outcome {
    edges::run(cx, query, references)
}

/// Check source capabilities before resolving inputs or opening a store.
pub(crate) fn unsupported(
    query: &ArticleSearch,
) -> Option<(biomcp_types::Source, biomcp_types::ArticleFilter)> {
    use biomcp_types::{ArticleFilter as F, ArticleSource as S};
    if query.sources().contains(&S::Litvar) {
        let variant = query.variant().map_or("", Text::as_str);
        let rsid = biomcp_adapters::litvar::is_rsid(variant);
        if variant.is_empty() || (!rsid && query.gene().is_none()) {
            return Some((biomcp_types::Source::LitVar, F::Variant));
        }
        if let Some(option) = query.rejects(&[
            F::Text,
            F::Disease,
            F::Drug,
            F::Author,
            F::Journal,
            F::Since,
            F::Until,
            F::Type,
            F::DateSort,
            F::OpenAccess,
            F::NoPreprints,
            F::ExcludeRetracted,
        ]) {
            return Some((biomcp_types::Source::LitVar, option));
        }
    }
    for (selector, source) in [
        (S::Litsense, biomcp_types::Source::LitSense),
        (S::Pubtator, biomcp_types::Source::PubTator),
    ] {
        if query.sources().contains(&selector)
            && let Some(option) = query.rejects(&[
                F::Author,
                F::Journal,
                F::Since,
                F::Until,
                F::Type,
                F::DateSort,
                F::OpenAccess,
                F::NoPreprints,
                F::ExcludeRetracted,
            ])
        {
            return Some((source, option));
        }
    }
    if query.sources().contains(&S::SemanticScholar) {
        return query
            .rejects(&[
                F::Author,
                F::Journal,
                F::Type,
                F::DateSort,
                F::NoPreprints,
                F::ExcludeRetracted,
            ])
            .map(|filter| (biomcp_types::Source::SemanticScholar, filter));
    }
    None
}
