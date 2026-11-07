//! Transport-owned SQLite schema. Other owners and tables remain untouched.
use crate::{CacheError, Deadline};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
mod admission;
#[cfg(test)]
mod contention;
mod schema;
pub use schema::{SchemaOwner, check_owners};
pub(crate) use schema::{check_current_owners, check_predecessor, db_error};
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded transport state across the request owners named by this module"
)]
use std::{
    path::Path,
    sync::{Mutex, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Transport SQLite schema version checked by other owners of the shared database.
pub const VERSION: u32 = 5;
/// Compatible cache namespace used by callers when naming shared cache files.
pub const CACHE_NAMESPACE_VERSION: u32 = 1;
const OWNER: &str = "biomcp-fetch";
/// Retain expired replies for stale and offline fallback before automatic pruning.
pub const STALE_GRACE_SECONDS: i64 = 30 * 24 * 60 * 60;
const OPEN_WAIT: Duration = Duration::from_secs(5);

// One connection is shared by source legs. Its lock covers only SQLite work;
// callers decode and check owned bytes after the guard has been released.
#[derive(Debug)]
#[expect(
    clippy::disallowed_types,
    reason = "Serialize only SQL on the connection shared by transport legs"
)]
pub(crate) struct Cache(
    Mutex<Connection>,
    pub(crate) std::path::PathBuf,
    pub(crate) std::path::PathBuf,
    Mutex<()>,
    RwLock<()>,
);

#[derive(Debug, serde::Serialize)]
pub(crate) struct Entry {
    pub(crate) status: u16,
    #[serde(skip)] // Containment scans raw and decoded body bytes directly.
    pub(crate) body: Vec<u8>,
    pub(crate) content_type: Option<String>,
    pub(crate) content_range: Option<String>,
    pub(crate) pagination: Option<crate::Pagination>,
    pub(crate) fresh_until: i64,
    pub(crate) fetched_at: i64,
}

impl Entry {
    pub(crate) fn new(status: u16, body: Vec<u8>, seconds: u64, content_type: &str) -> Self {
        let fetched_at = now();
        let fresh_until = fetched_at.saturating_add(i64::try_from(seconds).unwrap_or(i64::MAX));
        Self {
            status,
            body,
            content_range: None,
            pagination: None,
            content_type: (status != 404).then(|| content_type.to_owned()),
            fetched_at,
            fresh_until,
        }
    }
}

impl Cache {
    pub(crate) fn owned_connection(&self) -> Result<Connection, CacheError> {
        let connection = Connection::open_with_flags(
            &self.2,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|error| db_error(error))?;
        let version = connection
            .query_row("SELECT version FROM meta WHERE owner=?1", [OWNER], |row| {
                row.get::<_, u32>(0)
            })
            .optional()
            .map_err(|error| db_error(error))?;
        if version != Some(VERSION) {
            return Err(CacheError::Version);
        }
        Ok(connection)
    }
    #[expect(
        clippy::disallowed_types,
        reason = "Serialize only SQL on the connection shared by transport legs"
    )]
    pub(crate) fn open_with_owners(
        path: &Path,
        owners: &[SchemaOwner],
    ) -> Result<Self, CacheError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
            {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(CacheError::Filesystem),
            }
        }
        let mut connection = Connection::open(path).map_err(|error| db_error(error))?;
        // One deadline covers the lock and every step. Setup is serialized across
        // processes, so waiting openers never hold up the one doing setup.
        let deadline = Deadline::start(OPEN_WAIT, None);
        let _setup = setup_lock(path, deadline)?;
        step("owners", &mut connection, deadline, |connection| {
            check_owners(connection, owners)
        })?;
        step("initialize", &mut connection, deadline, Self::initialize)?;
        // Ownership is validated before changing the file's journal mode.
        step("wal", &mut connection, deadline, |connection| {
            enable_wal(connection)
        })?;
        Ok(Self(
            Mutex::new(connection),
            path.with_extension("parts"),
            path.to_owned(),
            Mutex::new(()),
            RwLock::new(()),
        ))
    }

    // try_lock avoids an unbounded Rust mutex wait. SQLite's own busy handler
    // uses the remaining deadline after acquiring the connection.
    pub(crate) fn sql<T>(
        &self,
        deadline: Deadline<'_>,
        operation: impl FnOnce(&Connection) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        let connection = wait_mutex(&self.0, deadline, &deadline.local(), None)?;
        sql_deadline(&connection, deadline)?;
        operation(&connection)
    }

    // Other schema owners retain the same connection guard around their callback.
    // Optional transport reads may bypass this hold so network work can progress.
    pub(crate) fn admit<T>(
        &self,
        deadline: Deadline<'_>,
        operation: impl FnOnce() -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        let connection = wait_mutex(&self.0, deadline, &deadline.local(), None)?;
        sql_deadline(&connection, deadline)?;
        let _owner = admission::Owner::enter(&self.4, &connection, deadline)?;
        operation()
    }

    // Ordinary transport mutations take the connection before write admission.
    pub(crate) fn write_sql<T>(
        &self,
        deadline: Deadline<'_>,
        operation: impl FnOnce(&Connection) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        let wall_bound = deadline.local();
        let connection = wait_mutex(&self.0, deadline, &wall_bound, None)?;
        let _writer = wait_mutex(&self.3, deadline, &wall_bound, None)?;
        sql_deadline(&connection, deadline)?;
        operation(&connection)
    }

    // Publishers borrow only write admission, never the ordinary connection.
    pub(crate) fn publish_sql<T>(
        &self,
        deadline: Deadline<'_>,
        operation: impl FnOnce() -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        let _writer = wait_mutex(&self.3, deadline, &deadline.local(), None)?;
        operation()
    }

    pub(crate) fn read(
        &self,
        key: &[u8; 32],
        max_body: u64,
        deadline: Deadline<'_>,
        opportunistic: bool,
    ) -> Result<Option<Entry>, CacheError> {
        let limit = i64::try_from(max_body).unwrap_or(i64::MAX);
        let operation = |connection: &Connection| {
            let entry = connection
                .query_row(
                    "SELECT status, CASE WHEN length(body)<=?2 THEN body ELSE NULL END, fresh_until, fetched_at, \
                    content_type, content_range, pagination FROM reply WHERE key=?1",
                    params![key, limit],
                    |row| {
                        Ok((
                            row.get::<_, u16>(0)?,
                            row.get::<_, Option<Vec<u8>>>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                            row.get::<_, Option<String>>(4)?,
                            row.get::<_, Option<String>>(5)?,
                            row.get::<_, Option<String>>(6)?,
                        ))
                    },
                )
                .optional()
                .map_err(|error| db_error(error))?;
            match entry {
                Some((
                    status,
                    Some(body),
                    fresh_until,
                    fetched_at,
                    content_type,
                    content_range,
                    pagination,
                )) if (200..300).contains(&status) || status == 404 && body.is_empty() => {
                    let pagination = pagination
                        .map(|p| serde_json::from_str::<crate::Pagination>(&p))
                        .transpose();
                    match pagination {
                        Ok(pagination)
                            if pagination.as_ref().is_none_or(crate::Pagination::valid) =>
                        {
                            Ok(Some(Entry {
                                status,
                                body,
                                fresh_until,
                                fetched_at,
                                content_type,
                                content_range,
                                pagination,
                            }))
                        }
                        _ => {
                            self.delete(connection, key)?;
                            Ok(None)
                        }
                    }
                }
                Some(_) => {
                    self.delete(connection, key)?;
                    Ok(None)
                }
                None => Ok(None),
            }
        };
        if opportunistic {
            self.try_sql(deadline, operation)
        } else {
            self.sql(deadline, operation)
        }
    }

    // Sibling legs share this connection. Wait for their local SQL work within
    // the command budget; external SQLite contention still falls through at once.
    fn try_sql<T>(
        &self,
        deadline: Deadline<'_>,
        operation: impl FnOnce(&Connection) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        if deadline.expired() {
            return Err(CacheError::Busy);
        }
        let connection = wait_mutex(&self.0, deadline, &deadline.local(), Some(&self.4))?;
        connection
            .busy_timeout(Duration::ZERO)
            .map_err(|error| db_error(error))?;
        operation(&connection)
    }

    pub(crate) fn remove(&self, key: &[u8; 32], deadline: Deadline<'_>) -> Result<(), CacheError> {
        self.try_sql(deadline, |connection| self.delete(connection, key))
    }

    fn delete(&self, connection: &Connection, key: &[u8; 32]) -> Result<(), CacheError> {
        let _writer = try_mutex(&self.3)?;
        connection
            .execute("DELETE FROM reply WHERE key=?1", [key])
            .map_err(|error| db_error(error))?;
        Ok(())
    }

    pub(crate) fn write(
        &self,
        key: &[u8; 32],
        source: biomcp_types::Source,
        url: &str,
        entry: &Entry,
        deadline: Deadline<'_>,
    ) -> Result<(), CacheError> {
        self.write_sql(deadline, |connection| {
            let tx = connection.unchecked_transaction().map_err(|error| db_error(error))?;
            tx.execute("DELETE FROM reply WHERE fresh_until<=?1", [now().saturating_sub(STALE_GRACE_SECONDS)]).map_err(|error| db_error(error))?;
            tx.execute(
                "INSERT INTO reply(key,source,url,status,content_type,body,fetched_at,fresh_until,content_range,pagination) \
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
                ON CONFLICT(key) DO UPDATE SET \
        source=excluded.source,url=excluded.url,status=excluded.status,content_type=excluded.content_type,body=excluded.body,\
                    fetched_at=excluded.fetched_at,fresh_until=excluded.fresh_until,content_range=excluded.content_range,pagination=excluded.pagination",
                params![
                    key,
                    source.to_string(),
                    url,
                    entry.status,
                    entry.content_type,
                    entry.body,
                    entry.fetched_at,
                    entry.fresh_until,
                    entry.content_range,
                    entry
                        .pagination
                        .as_ref()
                        .map(serde_json::to_string)
                        .transpose()
                        .map_err(|_| CacheError::Encoding)?
                ],
            )
            .map_err(|error| db_error(error))?;
            tx.commit().map_err(|error| db_error(error))
        })
    }
}

// Hold the sibling lock only while the file is set up. A file system without
// locks proceeds unlocked, and SQLite's own wait then applies.
fn setup_lock(path: &Path, deadline: Deadline<'_>) -> Result<Option<std::fs::File>, CacheError> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let Ok(file) = options.open(path.with_extension("open-lock")) else {
        return Ok(None);
    };
    let started = std::time::Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(Some(file)),
            Err(std::fs::TryLockError::WouldBlock) if !deadline.expired() => {
                std::thread::park_timeout(Duration::from_millis(1).min(deadline.remaining()));
            }
            Err(std::fs::TryLockError::WouldBlock) => {
                crate::observe::record_open_failure("lock", 0, started);
                return Err(CacheError::Busy);
            }
            Err(std::fs::TryLockError::Error(_)) => return Ok(None),
        }
    }
}

// Run one open step under the shared deadline; a Busy result is noted for proof runs.
fn step(
    name: &str,
    connection: &mut Connection,
    deadline: Deadline<'_>,
    operation: impl FnOnce(&mut Connection) -> Result<(), CacheError>,
) -> Result<(), CacheError> {
    let started = std::time::Instant::now();
    sql_deadline(connection, deadline)?;
    let result = operation(connection);
    if result == Err(CacheError::Busy) {
        crate::observe::record_open_failure(name, 0, started);
    }
    result
}

fn enable_wal(connection: &Connection) -> Result<(), CacheError> {
    let journal: String = connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .map_err(|error| db_error(error))?;
    if !journal.eq_ignore_ascii_case("wal") {
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| db_error(error))?;
    }
    Ok(())
}

// Required writers reserve admission before reading a snapshot they will mutate.
pub(crate) fn write_transaction(
    connection: &Connection,
) -> Result<rusqlite::Transaction<'_>, CacheError> {
    rusqlite::Transaction::new_unchecked(connection, rusqlite::TransactionBehavior::Immediate)
        .map_err(|error| db_error(error))
}

/// Configure SQLite's lock wait from the deadline; replay clocks never wait on OS locks.
pub fn sql_deadline(connection: &Connection, deadline: Deadline<'_>) -> Result<(), CacheError> {
    if deadline.expired() {
        return Err(CacheError::Busy);
    }
    connection
        .busy_timeout(if deadline.virtual_time() {
            Duration::ZERO
        } else {
            deadline.remaining()
        })
        .map_err(|error| db_error(error))
}
#[expect(
    clippy::disallowed_types,
    reason = "Serialize only SQL on the connection shared by transport legs"
)]
fn try_mutex<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, CacheError> {
    mutex.try_lock().map_err(|error| match error {
        std::sync::TryLockError::WouldBlock => CacheError::Busy,
        std::sync::TryLockError::Poisoned(_) => CacheError::Internal,
    })
}
#[expect(
    clippy::disallowed_types,
    reason = "Serialize only SQL on the connection shared by transport legs"
)]
fn wait_mutex<'a, T>(
    mutex: &'a Mutex<T>,
    deadline: Deadline<'_>,
    wall_bound: &crate::LocalDeadline,
    admission: Option<&RwLock<()>>,
) -> Result<std::sync::MutexGuard<'a, T>, CacheError> {
    loop {
        if deadline.expired() || wall_bound.expired() {
            return Err(CacheError::Busy);
        }
        match try_mutex(mutex) {
            Err(CacheError::Busy) => {
                if let Some(owner) = admission {
                    match owner.try_read() {
                        Ok(probe) => drop(probe),
                        Err(std::sync::TryLockError::WouldBlock) => return Err(CacheError::Busy),
                        Err(std::sync::TryLockError::Poisoned(_)) => {
                            return Err(CacheError::Internal);
                        }
                    }
                }
                std::thread::park_timeout(Duration::from_millis(1).min(deadline.remaining()))
            }
            result => return result,
        }
    }
}

pub(crate) fn now() -> i64 {
    timestamp().unwrap_or_default()
}

pub(crate) fn timestamp() -> Option<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
}
