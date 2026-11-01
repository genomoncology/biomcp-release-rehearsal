//! Retain complete source observations with their selectors and outcomes.
use crate::{Ctx, store::Store};
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::{Problem, SourceOutcome};
use serde::{Serialize, de::DeserializeOwned};
pub(crate) struct Saved<T> {
    pub dropped: bool,
    pub items: Vec<T>,
    pub sources: Vec<SourceOutcome>,
    pub problems: Vec<Problem>,
}
const VERSION: &str = "biomcp.source_snapshot/1";
type Wire<T> = (Vec<T>, Vec<SourceOutcome>, Vec<Problem>, bool);
pub(crate) fn key(namespace: &str, q: &impl Serialize) -> Result<Vec<u8>, RequestError> {
    serde_json::to_vec(&(VERSION, namespace, q)).map_err(|_| CacheError::Database.into())
}
pub(crate) fn load<T: DeserializeOwned>(
    cx: &Ctx<'_>,
    namespace: &str,
    q: &impl Serialize,
    pool: Option<&biodata::Text>,
    deadline: Deadline<'_>,
) -> Result<Option<Saved<T>>, RequestError> {
    let Some(id) = pool.filter(|_| !cx.fresh) else {
        return Ok(None);
    };
    let saved: Option<Wire<T>> = read(cx, &key(namespace, q)?, id.as_str(), deadline)?;
    Ok(saved.map(|(items, sources, problems, dropped)| Saved {
        items,
        sources,
        problems,
        dropped,
    }))
}
pub(crate) fn save<T: Serialize>(
    cx: &Ctx<'_>,
    namespace: &str,
    q: &impl Serialize,
    deadline: Deadline<'_>,
    saved: &Saved<T>,
) -> Result<biodata::Text, RequestError> {
    write(
        cx,
        &key(namespace, q)?,
        &(&saved.items, &saved.sources, &saved.problems, saved.dropped),
        &saved.sources,
        deadline,
    )
}

/// Load an entity-owned payload after source and caller admission.
pub(crate) fn read<T: DeserializeOwned>(
    cx: &Ctx<'_>,
    key: &[u8],
    token: &str,
    deadline: Deadline<'_>,
) -> Result<Option<T>, RequestError> {
    // Catalog pools supply their retained key; reject older formats before payload decoding.
    type Selector<'a> = (&'a str, serde::de::IgnoredAny, serde::de::IgnoredAny);
    if !serde_json::from_slice::<Selector<'_>>(key).is_ok_and(|(version, _, _)| version == VERSION)
    {
        return Ok(None);
    }
    let store = Store::open(cx.fetch.cache_access()?, deadline)?;
    store
        .source_snapshot(token, key, |s| cx.fetch.disabled(s))?
        .map(|payload| decode(&payload))
        .transpose()
}
fn decode<T: DeserializeOwned>(payload: &str) -> Result<T, RequestError> {
    serde_json::from_str(payload).map_err(|_| CacheError::Snapshot.into())
}
fn encode(payload: &impl Serialize) -> Result<String, RequestError> {
    serde_json::to_string(payload).map_err(|_| CacheError::Snapshot.into())
}
pub(crate) fn write(
    cx: &Ctx<'_>,
    key: &[u8],
    payload: &impl Serialize,
    sources: &[SourceOutcome],
    deadline: Deadline<'_>,
) -> Result<biodata::Text, RequestError> {
    let payload = encode(payload)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    Ok(store.save_source_snapshot(key, &payload, sources)?)
}
type Found<T> = (Option<(T, crate::store::Pool)>, bool);
/// Find the current retained native window, or validate its explicit continuation.
pub(crate) fn find<T: DeserializeOwned>(
    cx: &Ctx<'_>,
    key: &[u8],
    token: Option<&biodata::Text>,
    deadline: Deadline<'_>,
) -> Result<Found<T>, RequestError> {
    let store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let pool = store.find(key, token.map(biodata::Text::as_str))?;
    let mismatch = pool.as_ref().is_some_and(|p| p.key != key);
    let Some(pool) = pool.filter(|_| !mismatch) else {
        return Ok((None, mismatch));
    };
    let saved = store
        .source_snapshot(pool.info.id.as_str(), key, |s| cx.fetch.disabled(s))?
        .map(|payload| decode(&payload).map(|saved| (saved, pool)))
        .transpose()?;
    Ok((saved, mismatch))
}
pub(crate) fn publish(
    cx: &Ctx<'_>,
    key: &[u8],
    payload: &impl Serialize,
    size: u32,
    sources: &[SourceOutcome],
    pool: Option<crate::store::Pool>,
    deadline: Deadline<'_>,
) -> Result<crate::store::Pool, RequestError> {
    let payload = encode(payload)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    Ok(if let Some(pool) = pool {
        if store
            .source_snapshot(pool.info.id.as_str(), key, |s| cx.fetch.disabled(s))?
            .as_deref()
            == Some(payload.as_str())
        {
            return Ok(pool);
        }
        store.extend_source_pool(pool, &payload, size, sources)?
    } else {
        store.publish_source_pool(key, &payload, size, sources)?
    })
}

/// Reuse an identical complete collection, otherwise publish a new immutable pool.
pub(crate) fn intern(
    cx: &Ctx<'_>,
    key: &[u8],
    payload: &impl Serialize,
    size: u32,
    sources: &[SourceOutcome],
    deadline: Deadline<'_>,
) -> Result<crate::store::Pool, RequestError> {
    let payload = encode(payload)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    if let Some(pool) = store.find(key, None)?
        && pool.key == key
        && store
            .source_snapshot(pool.info.id.as_str(), key, |s| cx.fetch.disabled(s))?
            .as_deref()
            == Some(payload.as_str())
    {
        return Ok(pool);
    }
    Ok(store.publish_source_pool(key, &payload, size, sources)?)
}
