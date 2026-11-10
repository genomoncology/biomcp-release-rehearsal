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
    serde_json::to_vec(&(VERSION, namespace, q)).map_err(|_| CacheError::Encoding.into())
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
        .map(Option::flatten)
}
fn decode<T: DeserializeOwned>(payload: &str) -> Result<Option<T>, RequestError> {
    crate::store::labels::decode(payload).map_err(|_| CacheError::Snapshot.into())
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
/// Entity-owned membership completeness, independent of optional field problems.
pub(crate) trait Observation {
    fn whole(&self) -> bool;
}
impl<T, U> Observation for (T, U, bool) {
    fn whole(&self) -> bool {
        !self.2
    }
}
impl<T, U, V> Observation for (T, U, V, bool) {
    fn whole(&self) -> bool {
        !self.3
    }
}
pub(crate) fn current_key(key: &[u8]) -> Vec<u8> {
    [b"biomcp.source_current/2\0".as_slice(), key].concat()
}
pub(crate) fn sources_whole(sources: &[SourceOutcome]) -> bool {
    sources
        .iter()
        .all(|source| source.state == biomcp_types::SourceState::Ok)
}
type Found<T> = (Option<(T, crate::store::Pool)>, bool);
/// Find the current retained native window, or validate its explicit continuation.
pub(crate) fn find<T: DeserializeOwned + Observation>(
    cx: &Ctx<'_>,
    key: &[u8],
    token: Option<&biodata::Text>,
    deadline: Deadline<'_>,
) -> Result<Found<T>, RequestError> {
    let store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let found = store.find_source_pool(key, token.map(biodata::Text::as_str), |s| {
        cx.fetch.disabled(s)
    })?;
    let Some((pool, payload)) = found else {
        return Ok((None, false));
    };
    if pool.key != key {
        return Ok((None, true));
    }
    let saved = decode::<T>(&payload)?;
    Ok((
        saved
            .filter(|saved| {
                token.is_some() || saved.whole() && !pool.dropped && sources_whole(&pool.sources)
            })
            .map(|saved| (saved, pool)),
        false,
    ))
}

pub(crate) fn publish(
    cx: &Ctx<'_>,
    key: &[u8],
    payload: &(impl Serialize + Observation),
    size: u32,
    sources: &[SourceOutcome],
    pool: Option<crate::store::Pool>,
    deadline: Deadline<'_>,
) -> Result<crate::store::Pool, RequestError> {
    let incomplete = !payload.whole();
    let payload = encode(payload)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    Ok(if let Some(pool) = pool {
        store.extend_source_pool(pool, &payload, size, sources, incomplete)?
    } else {
        store.publish_source_pool(key, &payload, size, sources, incomplete)?
    })
}

/// Reuse an identical complete collection, otherwise publish a new immutable pool.
pub(crate) fn intern(
    cx: &Ctx<'_>,
    key: &[u8],
    payload: &(impl Serialize + Observation),
    size: u32,
    sources: &[SourceOutcome],
    deadline: Deadline<'_>,
) -> Result<crate::store::Pool, RequestError> {
    let incomplete = !payload.whole();
    let payload = encode(payload)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    if !incomplete
        && sources_whole(sources)
        && let Some((pool, retained)) =
            store.find_source_pool(key, None, |s| cx.fetch.disabled(s))?
        && pool.key == key
        && !pool.dropped
        && sources_whole(&pool.sources)
        && retained == payload
    {
        return Ok(pool);
    }
    Ok(store.publish_source_pool(key, &payload, size, sources, incomplete)?)
}
