//! PubMed snapshots and stable pages through one checked command consumer.
use biomcp_types::ReplyBody;
mod drug_names;
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
mod pool;
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
/// Sentences kept out of the store and given back to this reply.
type Held = Vec<(biomcp_types::ArticleKey, biomcp_types::ArticleSentence)>;
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let mut store = Store::open(cx.fetch.cache_access()?, *deadline)?;
    let bytes = serde_json::to_vec(&query.identity()).map_err(|_| CacheError::Encoding)?;
    let keys = pool::key(cx, b"biomcp.article_page/1\0", &bytes)?;
    let key = keys.identity;
    let explicit = query.pool().map(Text::as_str);
    let stored = if cx.fresh && explicit.is_none() {
        None
    } else {
        store.find(&keys.current, explicit)?
    };
    // A repeat search rechecks what an unfinished lookup left out; a named pool stays as paged.
    let restricted = cx.fetch.disabled_use(biomcp_types::Use::PerRecord);
    let stored = stored.filter(|pool| explicit.is_some() || !(restricted && pool.dropped));
    let (stored, looked_up) = drug_names::reuse(cx, query, stored, out, deadline);
    let publish = stored.is_none() && explicit.is_none();
    let (pool, held, whole) = match stored {
        Some(pool) if pool.key != key.as_slice() => {
            pool_error(query, true, out);
            return Ok(());
        }
        Some(pool) => (pool, Vec::new(), true),
        None if explicit.is_some() => {
            pool_error(query, false, out);
            return Ok(());
        }
        None => match fetch_pool(cx, query, &key, looked_up, &mut store, deadline, out)? {
            Some(fetched) => fetched,
            None => return Ok(()),
        },
    };
    out.sources = pool.sources.clone();
    let result = self::pool::read(
        cx,
        page::Cursor::Search(query),
        &mut store,
        &pool,
        out,
        keys.current.as_slice(),
        publish && whole,
    );
    if let ReplyData::ArticlePage {
        body: Some(page), ..
    } = &mut out.body
    {
        biomcp_adapters::article_rights::return_unchecked(&mut page.items, &held);
    }
    *deadline = store.deadline();
    result
}
/// Fetch and store a new pool. `None` means the reply is already complete in `out`.
fn fetch_pool<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    key: &[u8],
    looked_up: Option<crate::drug::search_names::Resolved>,
    store: &mut Store<'a>,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Option<(crate::store::Pool, Held, bool)>, RequestError> {
    let Some(resolved) = term::search(cx, query, deadline, out)? else {
        return Ok(None);
    };
    let drug_names =
        looked_up.unwrap_or_else(|| crate::drug::search_names::article(cx, query, out, deadline));
    let mut items = match sources::search(cx, query, &resolved, &drug_names.names, deadline, out)? {
        sources::SearchResult::Items(items) => items,
        sources::SearchResult::Answer => return Ok(None),
        sources::SearchResult::Unavailable => {
            retry(query, None, out);
            return Ok(None);
        }
    };
    if query.sort() == biomcp_types::ArticleSort::Citations
        && !order::apply(cx, deadline, &mut items, out)?
    {
        retry(query, None, out);
        return Ok(None);
    }
    filter_text(cx, deadline, &mut items, out);
    let held = biomcp_adapters::article_rights::take_unchecked(&mut items);
    store.set_deadline(*deadline);
    let Some(pool) = crate::publish::keep(
        store
            .publish(key, &items, &out.sources, &out.problems, out.dropped)
            .map_err(Into::into),
        out,
    )?
    else {
        biomcp_adapters::article_rights::return_unchecked(&mut items, &held);
        page::fetched(cx, deadline, page::Cursor::Search(query), items, out);
        return Ok(None);
    };
    // Names may be missing, so the pool serves this search's pages and never a new search.
    Ok(Some((pool, held, !drug_names.failed)))
}
/// Apply the record policy before publishing a newly fetched ranked pool.
fn filter_text<'a>(
    cx: &Ctx<'a>,
    deadline: &mut Deadline<'a>,
    items: &mut [biomcp_types::ArticleItem],
    out: &mut Outcome,
) {
    if cx.fetch.disabled_use(biomcp_types::Use::PerRecord) {
        let leg = cx.fetch.leg(&biomcp_adapters::pubmed::SPEC, *deadline);
        let started = leg.remaining();
        out.dropped |= biomcp_adapters::article_rights::items(&leg, items);
        *deadline = consume(*deadline, started, leg.remaining());
    }
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
