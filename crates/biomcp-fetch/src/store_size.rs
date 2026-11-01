//! Measure owned store files without attributing shared SQLite pages to bundles.
use crate::{CacheError, Deadline, Fetch};
use biomcp_types::StoreSize;
use rusqlite::{Connection, OpenFlags};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};
impl Fetch {
    /// Read file lengths only; do not synchronize, checkpoint, vacuum or print paths.
    pub fn store_size(&self, deadline: Deadline<'_>) -> Result<StoreSize, CacheError> {
        let Some(cache) = self.cache.as_ref() else {
            return Ok(StoreSize::default());
        };
        let mut result = StoreSize {
            active_bytes: database(&cache.2)?,
            partial_download_bytes: parts(&cache.1)?,
            ..StoreSize::default()
        };
        let mut seen = BTreeSet::from([cache.2.clone()]);
        // Replay and case-cache commands never inspect the ordinary user store.
        if self.config.replay_cache.is_none() {
            for path in retained_paths(&cache.2, &self.config.cache_predecessors)? {
                if deadline.expired() {
                    return Err(CacheError::Busy);
                }
                if !seen.insert(path.clone()) || regular_length(&path)?.is_none() {
                    continue;
                }
                let Ok(connection) =
                    Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                else {
                    continue;
                };
                connection
                    .busy_timeout(deadline.remaining())
                    .map_err(|_| CacheError::Database)?;
                if !owned(&connection, &self.config.cache_owners)? {
                    continue;
                }
                drop(connection);
                result.retained_bytes += database(&path)?;
                result.partial_download_bytes += parts(&path.with_extension("parts"))?;
            }
        }
        result.total_bytes =
            result.active_bytes + result.retained_bytes + result.partial_download_bytes;
        Ok(result)
    }
}
fn database(path: &Path) -> Result<u64, CacheError> {
    let mut bytes = regular_length(path)?.ok_or(CacheError::Filesystem)?;
    for suffix in ["-wal", "-shm"] {
        let mut name = path.as_os_str().to_os_string();
        name.push(suffix);
        bytes += regular_length(Path::new(&name))?.unwrap_or(0);
    }
    Ok(bytes)
}
fn regular_length(path: &Path) -> Result<Option<u64>, CacheError> {
    match fs::symlink_metadata(path) {
        Ok(meta) => Ok(meta.is_file().then_some(meta.len())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(CacheError::Filesystem),
    }
}
fn parts(path: &Path) -> Result<u64, CacheError> {
    match fs::symlink_metadata(path) {
        Ok(meta) if !meta.is_dir() => return Err(CacheError::Filesystem),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(_) => return Err(CacheError::Filesystem),
        Ok(_) => {}
    }
    let mut bytes = 0;
    for entry in fs::read_dir(path).map_err(|_| CacheError::Filesystem)? {
        let entry = entry.map_err(|_| CacheError::Filesystem)?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(crate::bundle::store::part_name)
        {
            bytes += regular_length(&entry.path())?.unwrap_or(0);
        }
    }
    Ok(bytes)
}

fn retained_paths(active: &Path, configured: &[PathBuf]) -> Result<Vec<PathBuf>, CacheError> {
    let mut paths = configured.to_vec();
    if let Some(folder) = active.parent() {
        for entry in fs::read_dir(folder).map_err(|_| CacheError::Filesystem)? {
            let entry = entry.map_err(|_| CacheError::Filesystem)?;
            if entry
                .file_name()
                .to_str()
                .and_then(crate::cleanup::namespace)
                .is_some()
            {
                paths.push(entry.path());
            }
        }
    }
    Ok(paths)
}
fn owned(connection: &Connection, owners: &[crate::SchemaOwner]) -> Result<bool, CacheError> {
    let read = || -> rusqlite::Result<Vec<String>> {
        connection
            .prepare("SELECT owner FROM meta")?
            .query_map([], |row| row.get(0))?
            .collect()
    };
    match read() {
        Ok(names) => Ok(names.iter().any(|name| name == "biomcp-fetch")
            && names.iter().all(|name| {
                name == "biomcp-fetch" || owners.iter().any(|owner| owner.name == name)
            })),
        Err(error)
            if matches!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
            ) =>
        {
            Err(CacheError::Busy)
        }
        Err(_) => Ok(false),
    }
}
