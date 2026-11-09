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
    for _ in 0..100 {
        crate::sqlite_diagnostic::append(&path, "fetch::cache.rs:376", &error);
    }
    crate::sqlite_diagnostic::append_open_failure(&path, "wal", 37, 50);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.len() <= 4096,
        "diagnostic output must retain its bound"
    );
    let last: serde_json::Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
    assert_eq!(
        last,
        serde_json::json!({"operation":"open.wal", "retries":37, "elapsed_ms":50})
    );
}

#[test]
fn first_open_waits_for_the_setup_lock_then_succeeds() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let holder = std::fs::File::create(path.with_extension("open-lock")).unwrap();
    holder.lock().unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        drop(holder);
    });
    let started = std::time::Instant::now();
    Cache::open_with_owners(&path, &[]).expect("open must wait for the setup lock");
    assert!(started.elapsed() >= Duration::from_millis(100));
    release.join().unwrap();
}

#[test]
fn first_open_refuses_busy_when_the_setup_lock_outlasts_the_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let holder = std::fs::File::create(path.with_extension("open-lock")).unwrap();
    holder.lock().unwrap();
    let lock = super::setup_lock(&path, Deadline::start(Duration::from_millis(50), None));
    assert!(matches!(lock, Err(CacheError::Busy)));
    drop(holder);
    assert!(Cache::open_with_owners(&path, &[]).is_ok());
}

// The installed first-open witness exercises this boundary through Fetch::open.
// Here a real writer fixes the first failure state without a release timer.
#[test]
fn wal_retries_an_observed_busy_after_the_writer_releases() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let holder = Connection::open(&path).unwrap();
    holder
        .execute_batch("CREATE TABLE held(value); BEGIN IMMEDIATE")
        .unwrap();
    let connection = Connection::open(&path).unwrap();
    let mut attempts = 0;
    let deadline = Deadline::start(Duration::from_secs(15), None);
    let result = super::retry_wal(&connection, deadline, || {
        attempts += 1;
        let timeout: i64 = connection
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();
        assert!(
            timeout > 0,
            "the production busy handler must remain enabled"
        );
        let result = super::enable_wal(&connection, deadline);
        if attempts == 1 {
            assert_eq!(
                result,
                Err(CacheError::Busy),
                "fixture must hold the WAL transition"
            );
            holder.execute_batch("ROLLBACK").unwrap();
        }
        result
    });
    assert_eq!(result, Ok(()), "a released writer must not leave open busy");
    assert_eq!(attempts, 2, "WAL must be retried after its real Busy");
    let journal: String = connection
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();
    assert_eq!(journal, "wal");
}

#[test]
fn wal_retry_never_restarts_the_open_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let holder = Connection::open(&path).unwrap();
    holder
        .execute_batch("CREATE TABLE held(value); BEGIN IMMEDIATE")
        .unwrap();
    let connection = Connection::open(&path).unwrap();
    let deadline = Deadline::start(Duration::from_millis(50), None);
    let result = super::retry_wal(&connection, deadline, || {
        super::enable_wal(&connection, deadline)
    });
    assert_eq!(result, Err(CacheError::Busy));
    assert!(
        deadline.expired(),
        "WAL returned Busy before its shared budget expired"
    );
    holder.execute_batch("ROLLBACK").unwrap();
    assert_eq!(
        super::enable_wal(&connection, deadline),
        Err(CacheError::Busy),
        "an expired read budget must not begin a journal-mode update"
    );
}
