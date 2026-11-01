use crate::{
    Ctx,
    store::{Pool, Store},
};
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::{PathwayEvents, PathwayGenes, Problem, SourceOutcome};
pub(super) struct Saved {
    pub dropped: bool,
    pub genes: Option<PathwayGenes>,
    pub events: Option<PathwayEvents>,
    pub sources: Vec<SourceOutcome>,
    pub problems: Vec<Problem>,
}
impl Saved {
    pub(super) fn decode(payload: &str) -> Result<Self, RequestError> {
        type Wire = (
            Option<PathwayGenes>,
            Option<PathwayEvents>,
            Vec<SourceOutcome>,
            Vec<Problem>,
            bool,
        );
        let (genes, events, sources, problems, dropped): Wire =
            serde_json::from_str(payload).map_err(|_| CacheError::Snapshot)?;
        Ok(Self {
            dropped,
            genes,
            events,
            sources,
            problems,
        })
    }
    fn encode(&self) -> Result<String, RequestError> {
        serde_json::to_string(&(
            &self.genes,
            &self.events,
            &self.sources,
            &self.problems,
            self.dropped,
        ))
        .map_err(|_| CacheError::Snapshot.into())
    }
}
pub(super) fn key(id: &biodata::PathwayId, genes: bool) -> Vec<u8> {
    format!(
        "biomcp.pathway_relation/3\0{id}\0{}",
        if genes { "genes" } else { "events" }
    )
    .into_bytes()
}
pub(super) fn load(
    cx: &Ctx<'_>,
    id: &biodata::PathwayId,
    genes: bool,
    token: Option<&biodata::Text>,
    deadline: Deadline<'_>,
) -> Result<Option<(Saved, Pool)>, RequestError> {
    let store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let key = key(id, genes);
    let Some(pool) = store.find(&key, token.map(biodata::Text::as_str))? else {
        return Ok(None);
    };
    if pool.key != key {
        return Ok(None);
    }
    let Some(payload) =
        store.source_snapshot(pool.info.id.as_str(), &key, |s| cx.fetch.disabled(s))?
    else {
        return Ok(None);
    };
    Ok(Some((Saved::decode(&payload)?, pool)))
}
pub(super) fn save(
    cx: &Ctx<'_>,
    id: &biodata::PathwayId,
    genes: bool,
    deadline: Deadline<'_>,
    saved: &Saved,
) -> Result<Pool, RequestError> {
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let size = saved
        .genes
        .as_ref()
        .map(|g| g.items.len())
        .or_else(|| saved.events.as_ref().map(|e| e.items.len()))
        .unwrap_or(0) as u32;
    Ok(store.publish_source_pool(&key(id, genes), &saved.encode()?, size, &saved.sources)?)
}
