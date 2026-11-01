use crate::{
    Ctx, Outcome,
    store::{Pool, Store},
};
use biomcp_adapters::semantic_scholar::author::PapersPage;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::{AuthorPapersQuery, ReplyData, Request, SourceOutcome, StepKind};
#[derive(Debug)]
pub(super) struct Saved {
    pub pages: Vec<PapersPage>,
    pub source: SourceOutcome,
    pub key_needed: bool,
}
fn key(q: &AuthorPapersQuery) -> Result<Vec<u8>, RequestError> {
    serde_json::to_vec(&(
        "biomcp.author_papers_snapshot/2",
        q.id(),
        q.section(),
        q.limit(),
    ))
    .map_err(|_| CacheError::Database.into())
}
pub(super) fn load(
    cx: &Ctx<'_>,
    q: &AuthorPapersQuery,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(Saved, Pool)>, RequestError> {
    if cx.fresh {
        if q.pool().is_some() {
            restart(q, out, false);
        }
        return Ok(None);
    }
    let store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let expected = key(q)?;
    let pool = store.find(&expected, q.pool().map(biodata::Text::as_str))?;
    if pool.is_none() && q.pool().is_none() {
        return Ok(None);
    }
    let mismatch = pool.as_ref().is_some_and(|pool| pool.key != expected);
    if let Some(pool) = pool.filter(|_| !mismatch) {
        let payload = store.source_snapshot(pool.info.id.as_str(), &expected, |source| {
            cx.fetch.disabled(source)
        })?;
        if let Some(payload) = payload {
            let (pages, source, key_needed): (Vec<PapersPage>, SourceOutcome, bool) =
                serde_json::from_str(&payload).map_err(|_| CacheError::Snapshot)?;
            let saved = Saved {
                pages,
                source,
                key_needed,
            };
            if saved.pages.iter().any(|p| q.offset() == p.offset)
                || saved
                    .pages
                    .last()
                    .is_some_and(|p| p.next == Some(q.offset()))
            {
                return Ok(Some((saved, pool)));
            }
        }
    }
    restart(q, out, mismatch);
    Ok(None)
}
fn restart(q: &AuthorPapersQuery, out: &mut Outcome, mismatch: bool) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::new(
        StepKind::Restart,
        &[super::next::author(q.id())],
        super::run(Request::AuthorPapers(q.restart())),
    ));
}
pub(super) fn save(
    cx: &Ctx<'_>,
    q: &AuthorPapersQuery,
    deadline: Deadline<'_>,
    saved: &Saved,
    pool: Option<Pool>,
) -> Result<Pool, RequestError> {
    let payload = serde_json::to_string(&(&saved.pages, &saved.source, saved.key_needed))
        .map_err(|_| CacheError::Snapshot)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let size = saved.pages.iter().map(|p| p.data.len() as u32).sum();
    let sources = std::slice::from_ref(&saved.source);
    Ok(if let Some(pool) = pool {
        store.extend_source_pool(pool, &payload, size, sources)?
    } else {
        store.publish_source_pool(&key(q)?, &payload, size, sources)?
    })
}
