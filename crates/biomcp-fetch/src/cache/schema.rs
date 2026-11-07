//! Validate SQLite owners before initializing the transport schema.
use super::{Cache, CacheError, OWNER, VERSION};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior};
/// Metadata owned by another schema sharing the active cache.
#[derive(Clone, Copy, Debug)]
pub struct SchemaOwner {
    /// Stable schema owner name.
    pub name: &'static str,
    /// Expected schema version.
    pub version: u32,
    /// Older versions that this owner explicitly migrates in place.
    pub migratable_versions: &'static [u32],
    /// Tables that must not exist without this owner's metadata.
    pub tables: &'static [&'static str],
}
/// Validate declared owners in one snapshot without changing journal mode or tables.
/// Existing transactions remain owned by the caller.
pub fn check_owners(connection: &Connection, owners: &[SchemaOwner]) -> Result<(), CacheError> {
    check_owners_at(connection, "main", owners, None)
}
fn check_owners_at(
    connection: &Connection,
    schema: &str,
    owners: &[SchemaOwner],
    transport: Option<SchemaOwner>,
) -> Result<(), CacheError> {
    let validate = |connection: &Connection| {
        validate_owners(connection, schema, owners)?;
        if let Some(owner) = transport {
            validate_owners(connection, schema, &[owner])?;
        }
        Ok(())
    };
    if !connection.is_autocommit() {
        return validate(connection);
    }
    let transaction = Transaction::new_unchecked(connection, TransactionBehavior::Deferred)
        .map_err(|error| db_error(error))?;
    validate(&transaction)?;
    transaction.commit().map_err(|error| db_error(error))
}
fn validate_owners(
    connection: &Connection,
    schema: &str,
    owners: &[SchemaOwner],
) -> Result<(), CacheError> {
    let has_meta: bool = connection
        .query_row(
            &format!("SELECT EXISTS(SELECT 1 FROM {schema}.sqlite_master WHERE name='meta')"),
            [],
            |r| r.get(0),
        )
        .map_err(|error| db_error(error))?;
    for owner in owners {
        let version: Option<u32> = if has_meta {
            connection
                .query_row(
                    &format!("SELECT version FROM {schema}.meta WHERE owner=?1"),
                    [owner.name],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|error| version_error(error))?
        } else {
            None
        };
        if version.is_some_and(|v| v != owner.version && !owner.migratable_versions.contains(&v)) {
            return Err(CacheError::Version);
        }
        if version.is_none() {
            for table in owner.tables {
                let orphan: bool = connection
                    .query_row(
                        &format!(
                            "SELECT EXISTS(SELECT 1 FROM {schema}.sqlite_master WHERE name=?1)"
                        ),
                        [table],
                        |r| r.get(0),
                    )
                    .map_err(|error| db_error(error))?;
                if orphan {
                    return Err(CacheError::Version);
                }
            }
        }
    }
    Ok(())
}
#[track_caller]
fn version_error(error: rusqlite::Error) -> CacheError {
    match error {
        rusqlite::Error::FromSqlConversionFailure(..)
        | rusqlite::Error::IntegralValueOutOfRange(..)
        | rusqlite::Error::InvalidColumnType(..) => CacheError::Version,
        _ => db_error(error),
    }
}

const TRANSPORT_OWNER: SchemaOwner = SchemaOwner {
    name: OWNER,
    version: VERSION,
    migratable_versions: &[1, 2, 3, 4],
    tables: &[
        "reply",
        "reply_fresh",
        "bundle",
        "bundle_sync",
        "bundle_row",
        "bundle_key",
        "bundle_key_find",
        "bundle_words",
        "bundle_content",
        "bundle_rejection",
    ],
};
pub(crate) fn check_current_owners(
    connection: &Connection,
    owners: &[SchemaOwner],
) -> Result<(), CacheError> {
    check_owners_at(
        connection,
        "main",
        owners,
        Some(SchemaOwner {
            migratable_versions: &[],
            ..TRANSPORT_OWNER
        }),
    )
}
pub(crate) fn check_predecessor(
    connection: &Connection,
    owners: &[SchemaOwner],
) -> Result<(), CacheError> {
    check_owners_at(connection, "predecessor", owners, Some(TRANSPORT_OWNER))
}

impl Cache {
    pub(super) fn initialize(connection: &mut Connection) -> Result<(), CacheError> {
        let has_meta: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='meta')",
                [],
                |row| row.get(0),
            )
            .map_err(|error| db_error(error))?;
        if has_meta
            && connection
                .query_row("SELECT version FROM meta WHERE owner=?1", [OWNER], |row| {
                    row.get::<_, u32>(0)
                })
                .optional()
                .map_err(|error| version_error(error))?
                == Some(VERSION)
        {
            return Ok(());
        }
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| db_error(error))?;
        let meta_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='meta')",
                [],
                |r| r.get(0),
            )
            .map_err(|error| db_error(error))?;
        let version: Option<u32> = if meta_exists {
            tx.query_row("SELECT version FROM meta WHERE owner=?1", [OWNER], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|error| version_error(error))?
        } else {
            None
        };
        check_owners(&tx, &[TRANSPORT_OWNER])?;
        create_reply_schema(&tx, version)?;
        crate::bundle::store::initialize(&tx)?;
        tx.execute(
            "INSERT INTO meta(owner,version) VALUES (?1,?2) ON CONFLICT(owner) DO UPDATE SET version=excluded.version",
            (OWNER, VERSION),
        )
        .map_err(|error| db_error(error))?;
        tx.commit().map_err(|error| db_error(error))
    }
}

fn create_reply_schema(tx: &Transaction<'_>, version: Option<u32>) -> Result<(), CacheError> {
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (owner TEXT PRIMARY KEY, version INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS reply (
                key BLOB PRIMARY KEY, source TEXT NOT NULL, url TEXT NOT NULL,
                status INTEGER NOT NULL, content_type TEXT, content_range TEXT, pagination TEXT, body BLOB NOT NULL,
                fetched_at INTEGER NOT NULL, fresh_until INTEGER NOT NULL);
            CREATE INDEX IF NOT EXISTS reply_fresh ON reply (fresh_until);",
    )
    .map_err(|error| db_error(error))?;
    if version == Some(1) {
        tx.execute("ALTER TABLE reply ADD COLUMN content_range TEXT", [])
            .map_err(|error| db_error(error))?;
    }
    if matches!(version, Some(1 | 2)) {
        tx.execute("ALTER TABLE reply ADD COLUMN pagination TEXT", [])
            .map_err(|error| db_error(error))?;
    }
    Ok(())
}

#[track_caller]
pub(crate) fn db_error(error: rusqlite::Error) -> CacheError {
    crate::record_sqlite_error("fetch", &error);
    if matches!(
        error.sqlite_error_code(),
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
    ) {
        CacheError::Busy
    } else {
        CacheError::Database
    }
}
