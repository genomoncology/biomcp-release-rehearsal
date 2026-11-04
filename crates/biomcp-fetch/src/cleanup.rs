//! Explicit cleanup retains fresh replies, bundles and unrelated schemas.
use crate::{
    CacheError, Deadline, Fetch, SchemaOwner,
    cache::{db_error, now},
};
use rusqlite::{Connection, OpenFlags};
use std::{fs, path::Path};

impl Fetch {
    /// Remove expired transport replies only when the caller explicitly requests cleanup.
    pub fn clean_expired_replies(&self, deadline: Deadline<'_>) -> Result<u64, CacheError> {
        let cache = self.cache.as_ref().ok_or(CacheError::Internal)?;
        cache.write_sql(deadline, |c| {
            c.execute("DELETE FROM reply WHERE fresh_until<=?1", [now()])
                .map(|n| n as u64)
                .map_err(|error| db_error(error))
        })
    }
}
impl crate::CacheAccess {
    /// Remove older cache namespaces containing only declared owners and their known tables.
    /// Retained query layouts, future namespaces, symlinks and foreign tables are preserved.
    pub fn clean_incompatible_files(
        &self,
        owners: &[SchemaOwner],
        retained_query_namespaces: &[u32],
    ) -> Result<u64, CacheError> {
        let active = self.path();
        let Some(current) = active
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(namespace)
        else {
            return Ok(0);
        };
        let Some(parent) = active.parent() else {
            return Ok(0);
        };
        let mut removed = 0;
        for entry in fs::read_dir(parent).map_err(|_| CacheError::Filesystem)? {
            let entry = entry.map_err(|_| CacheError::Filesystem)?;
            let kind = entry.file_type().map_err(|_| CacheError::Filesystem)?;
            let name = entry.file_name();
            let Some(version) = name.to_str().and_then(namespace) else {
                continue;
            };
            if !kind.is_file()
                || version == current
                || version.0 == current.0 && retained_query_namespaces.contains(&version.1)
                || version.0 > current.0
                || version.1 > current.1
            {
                continue;
            }
            if owned_file(&entry.path(), owners)? {
                remove_database(&entry.path())?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}
pub(crate) fn namespace(name: &str) -> Option<(u32, u32)> {
    let (fetch, query) = name
        .strip_prefix("cache-")?
        .strip_suffix(".sqlite")?
        .split_once('-')?;
    Some((fetch.parse().ok()?, query.parse().ok()?))
}
fn owned_file(path: &Path, owners: &[SchemaOwner]) -> Result<bool, CacheError> {
    let connection = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };
    // A foreign schema in a legacy file remains its owner's data.
    let mut statement = match connection.prepare("SELECT owner FROM meta") {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };
    let declared = statement
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|error| db_error(error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| db_error(error))?;
    if declared.is_empty()
        || declared
            .iter()
            .any(|name| name != "biomcp-fetch" && !owners.iter().any(|o| o.name == name))
    {
        return Ok(false);
    }
    let mut tables = connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")
        .map_err(|error| db_error(error))?;
    let names = tables
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|error| db_error(error))?;
    for name in names {
        let name = name.map_err(|error| db_error(error))?;
        if name == "bundle" {
            // An empty predecessor can receive a publication after inspection. Keep its file.
            return Ok(false);
        }
        let transport =
            declared.iter().any(|owner| owner == "biomcp-fetch") && transport_table(&name);
        let query = owners.iter().any(|owner| {
            declared.iter().any(|name| name == owner.name) && owner.tables.contains(&name.as_str())
        });
        if name != "meta" && !transport && !query {
            return Ok(false);
        }
    }
    Ok(true)
}
fn transport_table(name: &str) -> bool {
    matches!(
        name,
        "reply"
            | "bundle"
            | "bundle_sync"
            | "bundle_row"
            | "bundle_key"
            | "bundle_content"
            | "bundle_rejection"
            | "bundle_words"
            | "bundle_words_data"
            | "bundle_words_idx"
            | "bundle_words_content"
            | "bundle_words_docsize"
            | "bundle_words_config"
    )
}

fn remove_database(path: &Path) -> Result<(), CacheError> {
    fs::remove_file(path).map_err(|_| CacheError::Filesystem)?;
    for suffix in ["-wal", "-shm"] {
        let mut sidecar = path.as_os_str().to_os_string();
        sidecar.push(suffix);
        match fs::remove_file(sidecar) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(CacheError::Filesystem),
        }
    }
    Ok(())
}
