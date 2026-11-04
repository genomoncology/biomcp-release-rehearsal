//! Real SQLite locking cannot be driven by the provider's virtual clock.
//! This inner concurrency contract checks bounded refusal and retained data;
//! outside-in concurrent replacement exercises the complete publication path.
use super::{Cache, CacheError, Connection, Duration};
use crate::{Deadline, deadline::VirtualClock};

#[test]
fn discretionary_read_waits_for_held_local_sql_until_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let cache = Cache::open_with_owners(&directory.path().join("cache.sqlite"), &[]).unwrap();
    let owner = cache.0.lock().unwrap();
    let deadline = Deadline::start(Duration::from_millis(20), None);
    let wall_bound = deadline.local();
    assert_eq!(
        cache.try_sql(deadline, |_| -> Result<(), CacheError> {
            panic!("held local SQL became available")
        }),
        Err(CacheError::Busy)
    );
    assert!(
        wall_bound.expired(),
        "local contention returned before its bound"
    );
    drop(owner);
    assert_eq!(
        cache.try_sql(
            Deadline::start(Duration::from_secs(5), None),
            |connection| {
                let busy: i64 = connection
                    .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
                    .unwrap();
                assert_eq!(busy, 0, "external SQLite contention must remain immediate");
                Ok(42)
            }
        ),
        Ok(42)
    );
}

#[test]
fn discretionary_read_refuses_expired_or_cancelled_local_wait() {
    let directory = tempfile::tempdir().unwrap();
    let cache = Cache::open_with_owners(&directory.path().join("cache.sqlite"), &[]).unwrap();
    let _owner = cache.0.lock().unwrap();
    let (fetch, cancellation) = crate::Fetch::open(crate::FetchConfig::default())
        .unwrap()
        .cancellable()
        .unwrap();
    let cancelled = fetch.deadline(Duration::from_secs(5));
    cancellation.cancel();
    for deadline in [Deadline::start(Duration::ZERO, None), cancelled] {
        assert_eq!(
            cache.try_sql(deadline, |_| -> Result<(), CacheError> {
                panic!("expired local wait reached SQL")
            }),
            Err(CacheError::Busy)
        );
    }
}

#[test]
fn replay_admission_preserves_rows_under_an_external_writer() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let cache = Cache::open_with_owners(&path, &[]).unwrap();
    let owner = Connection::open(&path).unwrap();
    owner
        .execute_batch("CREATE TABLE admission(value TEXT); INSERT INTO admission VALUES('old'); BEGIN IMMEDIATE")
        .unwrap();
    let clock = VirtualClock::default();
    let deadline = Deadline::start(Duration::from_millis(5), Some(&clock));
    let replace = |connection: &Connection| {
        connection
            .execute("UPDATE admission SET value='new'", [])
            .map_err(super::db_error)
    };
    // The external transaction remains held until the attempt returns. There
    // is no timer that releases it and no assertion about machine speed.
    assert!(matches!(
        cache.sql(deadline, replace),
        Err(CacheError::Busy)
    ));
    assert!(
        !deadline.expired(),
        "disk admission must not advance replay time"
    );
    assert_eq!(
        owner
            .query_row("SELECT value FROM admission", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "old"
    );
    owner.execute_batch("ROLLBACK").unwrap();
    assert_eq!(cache.sql(deadline, replace).unwrap(), 1);
    assert_eq!(
        owner
            .query_row("SELECT value FROM admission", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "new"
    );
    assert!(matches!(
        cache.sql(Deadline::start(Duration::ZERO, Some(&clock)), replace),
        Err(CacheError::Busy)
    ));
}

#[test]
fn required_write_reserves_before_snapshot_and_preserves_existing_reader() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let cache = Cache::open_with_owners(&path, &[]).unwrap();
    let reader = Connection::open(&path).unwrap();
    reader
        .execute_batch(
            "CREATE TABLE admission(value TEXT); INSERT INTO admission VALUES('old'); BEGIN",
        )
        .unwrap();
    let value = |connection: &Connection| {
        connection
            .query_row("SELECT value FROM admission", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap()
    };
    assert_eq!(value(&reader), "old");
    let publisher = cache.owned_connection().unwrap();
    let transaction = super::write_transaction(&publisher).unwrap();
    assert_eq!(value(&transaction), "old");
    // This write occurs after the publisher's first read and before its first
    // mutation. Deferred admission lets it commit and invalidates that snapshot.
    let competitor = Connection::open(&path).unwrap();
    competitor.busy_timeout(Duration::ZERO).unwrap();
    let competing = competitor.execute("UPDATE admission SET value='competing'", []);
    transaction
        .execute("UPDATE admission SET value='new'", [])
        .expect("required writer must not upgrade a stale WAL snapshot");
    transaction.commit().unwrap();
    assert!(matches!(
        competing.map_err(super::db_error),
        Err(CacheError::Busy)
    ));
    assert_eq!(value(&reader), "old");
    reader.execute_batch("COMMIT").unwrap();
    assert_eq!(value(&reader), "new");
}

#[test]
fn schema_admission_clears_ownership_before_poison_is_observed() {
    let directory = tempfile::tempdir().unwrap();
    let cache = Cache::open_with_owners(&directory.path().join("cache.sqlite"), &[]).unwrap();
    let deadline = || Deadline::start(Duration::from_secs(5), None);
    assert_eq!(
        cache.admit(deadline(), || {
            assert!(matches!(
                cache.4.try_read(),
                Err(std::sync::TryLockError::WouldBlock)
            ));
            Ok(42)
        }),
        Ok(42)
    );
    assert!(cache.4.try_read().is_ok());
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cache.admit(deadline(), || -> Result<(), CacheError> {
            panic!("schema owner panic");
        })
    }));
    assert!(failed.is_err());
    assert!(matches!(
        cache.4.try_read(),
        Err(std::sync::TryLockError::Poisoned(_))
    ));
    assert_eq!(
        cache.try_sql(deadline(), |_| Ok(())),
        Err(CacheError::Internal)
    );
}

// This inner test owns the diagnostic privacy and nonfatal I/O boundary.
#[test]
fn sqlite_diagnostic_records_exclude_error_text_and_ignore_unavailable_files() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("sqlite.jsonl");
    let error = rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT_TRIGGER),
        Some("private SQL, binding and database path".into()),
    );
    crate::sqlite_diagnostic::append(&path, "fixture.insert", &error);
    assert!(!path.exists(), "diagnostics must not create a destination");
    std::fs::write(&path, []).unwrap();
    crate::sqlite_diagnostic::append(&path, "fixture.insert", &error);
    let record: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        record,
        serde_json::json!({"operation":"fixture.insert",
        "primary_code":19,"extended_code":1811})
    );
    crate::sqlite_diagnostic::append(root.path(), "fixture.insert", &error);
}
