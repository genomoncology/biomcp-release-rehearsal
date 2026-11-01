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
type Wire<T> = (Vec<T>, Vec<SourceOutcome>, Vec<Problem>, bool);
fn key(namespace: &str, q: &impl Serialize) -> Result<Vec<u8>, RequestError> {
    serde_json::to_vec(&(namespace, q)).map_err(|_| CacheError::Database.into())
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
    let store = Store::open(cx.fetch.cache_access()?, deadline)?;
    store
        .source_snapshot(id.as_str(), &key(namespace, q)?, |s| cx.fetch.disabled(s))?
        .map(|p| {
            serde_json::from_str::<Wire<T>>(&p)
                .map(|(items, sources, problems, dropped)| Saved {
                    dropped,
                    items,
                    sources,
                    problems,
                })
                .map_err(|_| CacheError::Snapshot.into())
        })
        .transpose()
}
pub(crate) fn save<T: Serialize>(
    cx: &Ctx<'_>,
    namespace: &str,
    q: &impl Serialize,
    deadline: Deadline<'_>,
    saved: &Saved<T>,
) -> Result<biodata::Text, RequestError> {
    let payload =
        serde_json::to_string(&(&saved.items, &saved.sources, &saved.problems, saved.dropped))
            .map_err(|_| CacheError::Snapshot)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    Ok(store.save_source_snapshot(&key(namespace, q)?, &payload, &saved.sources)?)
}
