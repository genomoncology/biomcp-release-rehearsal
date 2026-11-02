//! Per-command SQLite owner for fixed search snapshots.
#![expect(
    clippy::disallowed_methods,
    reason = "Keep wall time and SQLite state inside the reviewed store boundary"
)]
mod cleanup;
pub use cleanup::clean;
mod evidence;
mod trial;
pub(crate) use evidence::EvidenceState;
mod ownership;
mod pool;
use biomcp_fetch::{CacheAccess, CacheError, Deadline, SchemaOwner};
pub(crate) use pool::{Pool, StoredItem};
use rusqlite::{Connection, OptionalExtension, params};
#[expect(
    clippy::disallowed_types,
    reason = "Bound command-owned SQLite work with its wall clock and stable snapshot timestamp"
)]
use std::time::{Instant, SystemTime, UNIX_EPOCH};
/// Query schema version migrated independently of the cache filename.
pub(crate) const VERSION: u32 = 6;
/// Stable shared-file generation, independent of the query schema version.
pub const CACHE_NAMESPACE_VERSION: u32 = 5;
/// Supported file generations retained until a separate transfer verifies their contents.
pub const CACHE_NAMESPACES: &[u32] = &[CACHE_NAMESPACE_VERSION, 4];
/// Query-owned metadata checked by transport before any journal or schema mutation.
pub const OWNER: SchemaOwner = SchemaOwner {
    name: "biomcp-query",
    version: VERSION,
    migratable_versions: &[4, 5],
    tables: &[
        "pool",
        "pool_current",
        "pool_item",
        "citation_evidence",
        "trial_snapshot",
    ],
};
/// One command's owned connection, retaining the replay directory capability.
#[derive(Debug)]
#[expect(
    clippy::disallowed_types,
    reason = "Bound command-owned SQLite work with its wall clock and stable snapshot timestamp"
)]
pub(crate) struct Store<'a> {
    connection: Connection,
    deadline: Deadline<'a>,
    wall_end: Instant,
    _access: CacheAccess,
}
impl<'a> Store<'a> {
    #[expect(
        clippy::disallowed_types,
        reason = "Bound command-owned SQLite work with its wall clock and stable snapshot timestamp"
    )]
    pub(crate) fn open(access: CacheAccess, deadline: Deadline<'a>) -> Result<Self, CacheError> {
        if deadline.expired() {
            return Err(CacheError::Busy);
        }
        let started = Instant::now();
        let wall_end = started.checked_add(deadline.remaining()).unwrap_or(started);
        let mut connection = access.connect()?;
        connection
            .busy_timeout(deadline.remaining())
            .map_err(|error| db(error))?;
        access.admit(deadline, || {
            biomcp_fetch::check_owners(
                &connection,
                &[
                    SchemaOwner {
                        name: "biomcp-fetch",
                        version: biomcp_fetch::VERSION,
                        migratable_versions: &[],
                        tables: &["reply", "bundle"],
                    },
                    OWNER,
                ],
            )?;
            let version: Option<u32> = connection
                .query_row("SELECT version FROM meta WHERE owner=?1", [OWNER.name], |r| r.get(0))
                .optional()
                .map_err(|error| db(error))?;
            if version == Some(VERSION) {
                return Ok(());
            }
            let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate).map_err(|error| db(error))?;
            biomcp_fetch::check_owners(&tx, &[OWNER])?;
            let version: Option<u32> = tx
                .query_row("SELECT version FROM meta WHERE owner=?1", [OWNER.name], |r| r.get(0))
                .optional()
                .map_err(|error| db(error))?;
            if version.is_some_and(|version| version != VERSION) {
                for table in OWNER.tables.iter().rev() {
                    tx.execute_batch(&format!("DROP TABLE IF EXISTS {table}")).map_err(|error| db(error))?;
                }
            }
            tx.execute_batch(include_str!("schema.sql")).map_err(|error| db(error))?;
            tx.execute(
                "INSERT INTO meta(owner,version) VALUES (?1,?2) ON CONFLICT(owner) DO UPDATE SET version=excluded.version",
                params![OWNER.name, VERSION],
            )
            .map_err(|error| db(error))?;
            tx.commit().map_err(|error| db(error))?;
            connection.pragma_update(None, "journal_mode", "WAL").map_err(|error| db(error))?;
            Ok(())
        })?;
        Ok(Self {
            connection,
            deadline,
            wall_end,
            _access: access,
        })
    }
    pub(super) fn prepare(&self) -> Result<(), CacheError> {
        if self.deadline().expired() {
            return Err(CacheError::Busy);
        }
        self.connection
            .busy_timeout(self.deadline().remaining())
            .map_err(|error| db(error))
    }
    pub(super) fn read<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.prepare()?;
        self._access.admit(self.deadline(), || {
            if !self._access.is_hosted() {
                return operation(&self.connection);
            }
            let tx = self
                .connection
                .unchecked_transaction()
                .map_err(|error| db(error))?;
            let result = operation(&tx)?;
            tx.commit().map_err(|error| db(error))?;
            Ok(result)
        })
    }
    pub(super) fn write<T>(
        &mut self,
        operation: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.prepare()?;
        let deadline = self.deadline();
        self._access.admit(deadline, || {
            let tx = self
                .connection
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                .map_err(|error| db(error))?;
            let result = operation(&tx)?;
            tx.commit().map_err(|error| db(error))?;
            Ok(result)
        })
    }
    pub(crate) fn set_deadline(&mut self, deadline: Deadline<'a>) {
        self.deadline = deadline;
    }
    #[expect(
        clippy::disallowed_types,
        reason = "Bound command-owned SQLite work with its wall clock and stable snapshot timestamp"
    )]
    pub(crate) fn deadline(&self) -> Deadline<'a> {
        self.deadline
            .child(self.wall_end.saturating_duration_since(Instant::now()))
    }
    #[expect(
        clippy::disallowed_types,
        reason = "Bound command-owned SQLite work with its wall clock and stable snapshot timestamp"
    )]
    pub(crate) fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|t| i64::try_from(t.as_secs()).unwrap_or(i64::MAX))
            .unwrap_or(0)
    }
}
#[track_caller]
fn db(error: rusqlite::Error) -> CacheError {
    biomcp_fetch::record_sqlite_error("query.store", &error);
    if matches!(
        error.sqlite_error_code(),
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
    ) {
        CacheError::Busy
    } else {
        CacheError::Database
    }
}

/// Validate and open the owned query store without making a source request.
pub fn health(fetch: &biomcp_fetch::Fetch, budget: std::time::Duration) -> Result<(), CacheError> {
    let access = fetch.cache_access()?;
    let connection = access.connect_read_only()?;
    let deadline = fetch.deadline(budget);
    connection
        .busy_timeout(deadline.remaining())
        .map_err(|error| db(error))?;
    access.admit(deadline, || {
        biomcp_fetch::check_owners(
            &connection,
            &[SchemaOwner {
                migratable_versions: &[],
                ..OWNER
            }],
        )
    })
}

#[cfg(test)]
mod privacy_tests;
