use crate::{CacheError, Fetch, ReplayCache};
use std::{path::PathBuf, sync::Arc};
/// Capability for another reviewed schema owner to open the active cache.
#[derive(Clone)]
pub struct CacheAccess {
    path: PathBuf,
    _cache: Arc<crate::cache::Cache>,
    _replay: Option<ReplayCache>,
    hosted: bool,
    secrets: Arc<crate::Secrets>,
}
impl std::fmt::Debug for CacheAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CacheAccess")
    }
}
impl CacheAccess {
    /// Whether retained licensed data requires caller ownership admission.
    pub fn is_hosted(&self) -> bool {
        self.hosted
    }
    /// Scope retained data without exposing its effective credential identity.
    pub fn namespace(&self, name: crate::SecretName, identity: [u8; 32]) -> Option<[u8; 32]> {
        self.secrets
            .fingerprint(name)
            .map(|key| key.namespace(identity))
    }
    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }
    /// Serialize SQL across schema owners sharing this Fetch, without holding admission during HTTP.
    pub fn admit<T>(
        &self,
        deadline: crate::Deadline<'_>,
        operation: impl FnOnce() -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self._cache.admit(deadline, operation)
    }

    /// Inspect the selected owned file without creating a database or journal.
    pub fn connect_read_only(&self) -> Result<rusqlite::Connection, CacheError> {
        rusqlite::Connection::open_with_flags(
            &self.path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|error| {
            crate::record_sqlite_error("query.connection", &error);
            CacheError::Database
        })
    }
    /// Open the selected owned file; callers validate their metadata before writes.
    pub fn connect(&self) -> Result<rusqlite::Connection, CacheError> {
        rusqlite::Connection::open(&self.path).map_err(|error| {
            crate::record_sqlite_error("query.connection", &error);
            CacheError::Database
        })
    }
}
impl Fetch {
    /// Retain the active reply cache and its directory owner for a query connection.
    pub fn cache_access(&self) -> Result<CacheAccess, CacheError> {
        let cache = self.cache.as_ref().ok_or(CacheError::Internal)?;
        Ok(CacheAccess {
            path: cache.2.clone(),
            _cache: cache.clone(),
            _replay: self.config.replay_cache.clone(),
            hosted: self.hosted,
            secrets: self.secrets.clone(),
        })
    }
}
