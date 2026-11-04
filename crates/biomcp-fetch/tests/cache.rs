//! Checked reply cache through the public transport API and documented SQLite schema.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "Fixture setup, child processes and failed source assertions must abort the contract test"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "Scoped callers exercise public cache behavior against controlled local peers"
)]
use biomcp_fetch::{Input, Read};
#[path = "support/cache_process.rs"]
mod cache_process;
mod support;
use biomcp_fetch::{
    Arg, BadReply, CacheError, Credential, Endpoint, Fetch, FetchConfig, FetchError, Freshness,
    KeyPlace, KeyUse, Mode, Quota, Rate, Reply, RequestError, Secret, SecretName, SourceKey,
    SourceSpec,
};
use biomcp_types::Reason;
use rusqlite::Connection;
use std::{path::Path, thread, time::Duration};
use support::Answer;
type Server = support::Server<()>;

fn spec(server: &Server, key: KeyUse) -> &'static SourceSpec {
    server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("cache"),
            Rate::per_second(1000),
            None,
            8,
        ))),
        key,
    )
}
fn config(path: &Path) -> FetchConfig {
    FetchConfig {
        cache: Some(path.to_owned()),
        ..FetchConfig::default()
    }
}
fn get(
    fetch: &Fetch,
    spec: &'static SourceSpec,
    endpoint: &Endpoint,
    params: &[(&str, Arg<'_>)],
) -> Result<Answer, RequestError> {
    fetch
        .leg(spec, fetch.deadline(Duration::from_secs(10)))
        .call(endpoint, Input::Params(params), Read::json())
        .map(|received| received.value)
}
fn exchange<T: Send>(
    server: &Server,
    status: u16,
    body: &str,
    call: impl FnOnce() -> T + Send,
) -> T {
    thread::scope(|scope| {
        let worker = scope.spawn(call);
        let request = server.next();
        assert!(
            request.text.starts_with("GET /answer"),
            "Cache admission must perform only the expected network exchange"
        );
        request.respond(status, "", body);
        worker.join().unwrap()
    })
}
fn expire(path: &Path) {
    Connection::open(path)
        .unwrap()
        .execute("UPDATE reply SET fresh_until=0", [])
        .unwrap();
}
fn source(error: RequestError) -> biomcp_fetch::SourceError {
    let RequestError::Source(error) = error else {
        panic!("expected provider failure: {error}");
    };
    error
}

#[path = "cache/credentials.rs"]
mod credentials;
#[path = "cache/freshness.rs"]
mod freshness;
#[path = "cache/storage.rs"]
mod storage;

const CHECKED_OWNER: biomcp_fetch::SchemaOwner = biomcp_fetch::SchemaOwner {
    name: "checked-owner",
    version: 1,
    migratable_versions: &[],
    tables: &["checked_data"],
};

struct Initialization {
    writer: Connection,
    result: Option<rusqlite::Result<()>>,
}

thread_local! {
    #[expect(clippy::disallowed_types, reason = "Each test thread owns its injected SQLite initialization state")]
    static INITIALIZATION: std::cell::RefCell<Option<Initialization>> = const {
        std::cell::RefCell::new(None)
    };
}

fn initialize_between_owner_reads(event: rusqlite::trace::TraceEvent<'_>) {
    let rusqlite::trace::TraceEvent::Stmt(_, sql) = event else {
        return;
    };
    if !sql.contains("sqlite_master WHERE name=?1") {
        return;
    }
    INITIALIZATION.with(|state| {
        if let Some(state) = state.borrow_mut().as_mut()
            && state.result.is_none()
        {
            state.result = Some(state.writer.execute_batch(
                "BEGIN IMMEDIATE;
                 CREATE TABLE meta(owner TEXT PRIMARY KEY, version INTEGER NOT NULL);
                 INSERT INTO meta VALUES('checked-owner',1);
                 CREATE TABLE checked_data(value TEXT);
                 COMMIT;",
            ));
        }
    });
}

#[test]
fn ownership_accepts_initialization_committed_between_metadata_and_table_reads() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("shared.sqlite");
    let writer = Connection::open(&path).unwrap();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    let reader = Connection::open(&path).unwrap();
    INITIALIZATION.with(|state| {
        *state.borrow_mut() = Some(Initialization {
            writer,
            result: None,
        });
    });
    reader.trace_v2(
        rusqlite::trace::TraceEventCodes::SQLITE_TRACE_STMT,
        Some(initialize_between_owner_reads),
    );
    let result = biomcp_fetch::check_owners(&reader, &[CHECKED_OWNER]);
    reader.trace_v2(rusqlite::trace::TraceEventCodes::empty(), None);
    INITIALIZATION.with(|state| {
        let initialization = state.borrow_mut().take().unwrap();
        initialization.result.unwrap().unwrap();
    });
    assert!(
        result.is_ok(),
        "valid concurrently initialized store: {result:?}"
    );
    assert!(reader.is_autocommit());
    assert!(biomcp_fetch::check_owners(&reader, &[CHECKED_OWNER]).is_ok());
}

#[test]
fn ownership_preserves_version_and_sql_error_contracts() {
    for (sql, expected) in [
        ("CREATE TABLE checked_data(value TEXT)", CacheError::Version),
        (
            "CREATE TABLE meta(owner TEXT,version); INSERT INTO meta VALUES('checked-owner',2)",
            CacheError::Version,
        ),
        (
            "CREATE TABLE meta(owner TEXT,version); INSERT INTO meta VALUES('checked-owner','bad')",
            CacheError::Version,
        ),
        (
            "CREATE TABLE meta(owner TEXT,version); INSERT INTO meta VALUES('checked-owner',-1)",
            CacheError::Version,
        ),
        (
            "CREATE TABLE meta(owner TEXT,version); INSERT INTO meta VALUES('checked-owner',NULL)",
            CacheError::Version,
        ),
        ("CREATE TABLE meta(owner TEXT)", CacheError::Database),
        ("CREATE TABLE meta(version INTEGER)", CacheError::Database),
    ] {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(sql).unwrap();
        let result = biomcp_fetch::check_owners(&connection, &[CHECKED_OWNER]);
        assert!(
            matches!(result, Err(ref error) if std::mem::discriminant(error) == std::mem::discriminant(&expected)),
            "{sql}: {result:?}"
        );
        assert!(connection.is_autocommit());
    }
}

#[test]
fn ownership_retains_caller_transaction_on_success_and_failure() {
    for (version, accepted) in [(1, true), (2, false)] {
        let mut connection = Connection::open_in_memory().unwrap();
        let transaction = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .unwrap();
        transaction
            .execute_batch("CREATE TABLE meta(owner TEXT,version INTEGER); CREATE TABLE checked_data(value TEXT);")
            .unwrap();
        transaction
            .execute("INSERT INTO meta VALUES('checked-owner',?1)", [version])
            .unwrap();
        assert_eq!(
            biomcp_fetch::check_owners(&transaction, &[CHECKED_OWNER]).is_ok(),
            accepted
        );
        assert!(!transaction.is_autocommit());
        transaction
            .execute("INSERT INTO checked_data VALUES('caller')", [])
            .unwrap();
        transaction.rollback().unwrap();
        let count: u32 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name IN ('meta','checked_data')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}

#[test]
fn ownership_version_query_reports_a_locked_table_as_busy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("locked.sqlite");
    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
        | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
        | rusqlite::OpenFlags::SQLITE_OPEN_SHARED_CACHE;
    let mut writer = Connection::open_with_flags(&path, flags).unwrap();
    writer
        .execute_batch("CREATE TABLE meta(owner TEXT PRIMARY KEY,version INTEGER); INSERT INTO meta VALUES('checked-owner',1);")
        .unwrap();
    let reader = Connection::open_with_flags(&path, flags).unwrap();
    let transaction = writer.transaction().unwrap();
    transaction
        .execute("UPDATE meta SET version=1", [])
        .unwrap();
    let result = biomcp_fetch::check_owners(&reader, &[CHECKED_OWNER]);
    assert!(matches!(result, Err(CacheError::Busy)), "{result:?}");
    assert!(reader.is_autocommit());
    transaction.rollback().unwrap();
}
