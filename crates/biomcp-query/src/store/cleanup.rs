//! Pool expiry shares publication; reply writes retain a stale grace window until pruning or explicit cleanup.
use super::{CacheError, Store, db};
use biomcp_types::StoreClean;
use rusqlite::{Transaction, params};
const PUBLISH_BATCH: i64 = 100;

pub(super) fn expired(tx: &Transaction<'_>, limit: i64) -> Result<StoreClean, CacheError> {
    let now = Store::now();
    let ids = {
        let mut statement = tx
            .prepare("SELECT id FROM pool WHERE expires<=?1 ORDER BY expires,id LIMIT ?2")
            .map_err(|error| db(error))?;
        statement
            .query_map(params![now, limit], |r| r.get::<_, String>(0))
            .map_err(|error| db(error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| db(error))?
    };
    let mut counts = StoreClean::default();
    for id in ids {
        counts.items += tx
            .execute("DELETE FROM pool_item WHERE pool=?1", [&id])
            .map_err(|error| db(error))? as u64;
        tx.execute("DELETE FROM pool_current WHERE pool=?1", [&id])
            .map_err(|error| db(error))?;
        counts.pools += tx
            .execute("DELETE FROM pool WHERE id=?1", [&id])
            .map_err(|error| db(error))? as u64;
    }
    counts.snapshots += tx
        .execute(
            "DELETE FROM trial_snapshot WHERE id IN (SELECT id FROM trial_snapshot WHERE expires<=?1 ORDER BY \
        expires,id LIMIT ?2)",
            params![now, limit],
        )
        .map_err(|error| db(error))? as u64;
    counts.snapshots += tx
        .execute(
            "DELETE FROM citation_evidence WHERE id IN (SELECT id FROM citation_evidence WHERE expires<=?1 \
        ORDER BY expires,id LIMIT ?2)",
            params![now, limit],
        )
        .map_err(|error| db(error))? as u64;
    Ok(counts)
}
impl Store<'_> {
    pub(super) fn collect_expired(tx: &Transaction<'_>) -> Result<(), CacheError> {
        expired(tx, PUBLISH_BATCH).map(|_| ())
    }
}
/// Remove expired owned state without making a source request. Fresh replies and bundles remain.
pub fn clean(
    fetch: &biomcp_fetch::Fetch,
    budget: std::time::Duration,
) -> Result<StoreClean, CacheError> {
    let deadline = fetch.deadline(budget);
    let mut store = Store::open(fetch.cache_access()?, deadline)?;
    let mut counts = store.write(|tx| expired(tx, -1))?;
    counts.replies = fetch.clean_expired_replies(deadline)?;
    counts.files = store
        ._access
        .clean_incompatible_files(&[super::OWNER], super::CACHE_NAMESPACES)?;
    Ok(counts)
}
