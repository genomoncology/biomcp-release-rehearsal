use super::*;
use biomcp_fetch::{Input, Read};

#[test]
fn fresh_hit_and_force_refresh_and_never_and_disabled_storage() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    assert!(
        exchange(&server, 200, r#"{"answer":true}"#, || get(
            &fetch,
            spec,
            &endpoint,
            &[]
        ))
        .unwrap()
        .answer
    );
    assert!(get(&fetch, spec, &endpoint, &[]).unwrap().answer);
    assert!(server.requests.try_recv().is_err());
    let forced = Fetch::open(FetchConfig {
        mode: Mode::Live { fresh: true },
        ..config(&path)
    })
    .unwrap();
    exchange(&server, 200, r#"{"answer":true,"refreshed":true}"#, || {
        get(&forced, spec, &endpoint, &[])
    })
    .unwrap();
    let never = spec.get("/answer");
    exchange(&server, 200, r#"{"answer":true,"never":true}"#, || {
        get(&fetch, spec, &never, &[])
    })
    .unwrap();
    let disabled = Fetch::open(FetchConfig::default()).unwrap();
    for _ in 0..2 {
        exchange(&server, 200, r#"{"answer":true}"#, || {
            get(&disabled, spec, &endpoint, &[])
        })
        .unwrap();
    }
    let rows: u32 = Connection::open(&path)
        .unwrap()
        .query_row("SELECT count(*) FROM reply", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1);
    let stored: String = Connection::open(&path)
        .unwrap()
        .query_row("SELECT CAST(body AS TEXT) FROM reply", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, r#"{"answer":true,"refreshed":true}"#);
}

#[test]
fn rejected_bodies_are_not_stored_and_negative_expiry_never_falls_back() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Minutes(30));
    for body in ["{broken", r#"{"answer":false}"#] {
        let error = exchange(&server, 200, body, || get(&fetch, spec, &endpoint, &[])).unwrap_err();
        assert_eq!(source(error).reason, Reason::BadReply);
        let rows: u32 = Connection::open(&path)
            .unwrap()
            .query_row("SELECT count(*) FROM reply", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 0);
    }
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    expire(&path);
    assert!(
        exchange(&server, 404, "unchecked-private-error-text", || fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| Some(received.value))
            .or_else(|error| match error {
                biomcp_fetch::RequestError::Source(source)
                    if source.reason == biomcp_types::Reason::NotFound =>
                    Ok(None),
                error => Err(error),
            }))
        .unwrap()
        .is_none()
    );
    // A negative hit returns absence without decoding the empty stored body.
    assert!(
        fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| Some(received.value))
            .or_else(|error| match error {
                biomcp_fetch::RequestError::Source(source)
                    if source.reason == biomcp_types::Reason::NotFound =>
                    Ok(None),
                error => Err(error),
            })
            .unwrap()
            .is_none()
    );
    let connection = Connection::open(&path).unwrap();
    let (status, body, ttl): (u16, Vec<u8>, i64) = connection
        .query_row(
            "SELECT status,body,fresh_until-fetched_at FROM reply",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((status, body.len(), ttl), (404, 0, 600));
    expire(&path);
    let error = exchange(&server, 400, "bad", || get(&fetch, spec, &endpoint, &[])).unwrap_err();
    assert_eq!(source(error).reason, Reason::Rejected);
}

#[test]
fn stale_fallback_retains_original_failure_across_later_success() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Days(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    expire(&path);
    let report = thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
            for _ in 0..3 {
                assert!(
                    leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                        .map(|received| received.value)
                        .unwrap()
                        .answer
                );
            }
            assert!(
                leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                    .map(|received| received.value)
                    .unwrap()
                    .answer
            );
            leg.finish()
        });
        for (status, body) in [
            (200, "{broken"),
            (200, r#"{"answer":false}"#),
            (400, "error"),
        ] {
            server.next().respond(status, "", body);
        }
        server.next().ok();
        worker.join().unwrap()
    });
    assert!(report.stale);
    assert_eq!(
        report
            .failures
            .iter()
            .map(|error| (error.reason, error.status))
            .collect::<Vec<_>>(),
        [
            (Reason::BadReply, Some(200)),
            (Reason::BadReply, Some(200)),
            (Reason::Rejected, Some(400))
        ]
    );
    expire(&path);
    let forced = Fetch::open(FetchConfig {
        mode: Mode::Live { fresh: true },
        ..config(&path)
    })
    .unwrap();
    assert_eq!(
        source(
            exchange(&server, 200, "{broken", || get(
                &forced,
                spec,
                &endpoint,
                &[]
            ))
            .unwrap_err()
        )
        .reason,
        Reason::BadReply
    );
}

#[test]
fn every_cached_read_obeys_current_reply_type_body_limit_and_sql_contract() {
    #[derive(Debug, serde::Deserialize)]
    struct Strict {
        answer: bool,
    }
    impl Reply for Strict {
        fn check(&self) -> Result<(), BadReply> {
            if self.answer { Err(BadReply) } else { Ok(()) }
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    // A stricter public type invalidates and replaces the previous checked body.
    let strict = exchange(&server, 200, r#"{"answer":false}"#, || {
        fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(&endpoint, Input::Params(&[]), Read::<Strict>::json())
            .map(|received| received.value)
    })
    .unwrap();
    assert!(!strict.answer);
    let limited = Fetch::open(FetchConfig {
        max_body: 1,
        ..config(&path)
    })
    .unwrap();
    // The oversized old entry is evicted. Its stale body cannot hide this failure.
    let error = exchange(&server, 400, "rejected", || {
        limited
            .leg(spec, limited.deadline(Duration::from_secs(10)))
            .call(&endpoint, Input::Params(&[]), Read::<Strict>::json())
            .map(|received| received.value)
    })
    .unwrap_err();
    assert_eq!(source(error).reason, Reason::Rejected);
    let count: u32 = Connection::open(&path)
        .unwrap()
        .query_row("SELECT count(*) FROM reply", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
    Connection::open(&path)
        .unwrap()
        .execute_batch("DROP TABLE reply")
        .unwrap();
    assert_eq!(
        get(&fetch, spec, &endpoint, &[]).unwrap_err(),
        RequestError::Cache(CacheError::Database)
    );
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn ownership_is_checked_before_any_schema_or_journal_change() {
    let directory = tempfile::tempdir().unwrap();
    for (name, sql) in [
        (
            "version",
            "CREATE TABLE meta(owner TEXT PRIMARY KEY,version INTEGER NOT NULL); INSERT INTO meta VALUES('biomcp-fetch',999);",
        ),
        (
            "orphan",
            "CREATE TABLE reply(untouched TEXT); INSERT INTO reply VALUES('keep');",
        ),
        (
            "foreign",
            "CREATE TABLE meta(owner TEXT PRIMARY KEY,version INTEGER NOT NULL); INSERT INTO meta VALUES('other',7); \
        CREATE TABLE reply(untouched TEXT);",
        ),
    ] {
        let path = directory.path().join(name);
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch(sql).unwrap();
        let before: Vec<String> = connection
            .prepare("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(matches!(
            Fetch::open(config(&path)),
            Err(FetchError::Cache(CacheError::Version))
        ));
        let after: Vec<String> = connection
            .prepare("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(
            connection
                .query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "delete"
        );
    }
    let path = directory.path().join("shared");
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE meta(owner TEXT PRIMARY KEY,version INTEGER NOT NULL); INSERT INTO meta \
        VALUES('other',7); CREATE TABLE other_data(value TEXT); INSERT INTO other_data VALUES('keep');",
        )
        .unwrap();
    Fetch::open(config(&path)).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT version FROM meta WHERE owner='other'", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        7
    );
    assert_eq!(
        connection
            .query_row("SELECT value FROM other_data", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
}

#[test]
fn version_one_cache_migrates_without_losing_other_owners_or_cached_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("migration.sqlite");
    drop(Fetch::open(config(&path)).unwrap());
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "ALTER TABLE reply DROP COLUMN pagination; ALTER TABLE reply DROP COLUMN content_range; UPDATE meta \
        SET version=1 WHERE owner='biomcp-fetch'; CREATE TABLE other_data(value TEXT); INSERT INTO \
        other_data VALUES('keep'); INSERT INTO meta VALUES('other',7); INSERT INTO \
        reply(key,source,url,status,content_type,body,fetched_at,fresh_until) \
        VALUES(zeroblob(32),'cpic','https://example.org/synthetic',200,'application/json','[]',1,2);",
        )
        .unwrap();
    drop(Fetch::open(config(&path)).unwrap());
    let value: (String, u32, Option<String>) = connection
        .query_row(
            "SELECT body,(SELECT version FROM meta WHERE owner='biomcp-fetch'),content_range FROM reply",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(value, ("[]".into(), biomcp_fetch::VERSION, None));
    assert_eq!(
        connection
            .query_row("SELECT value FROM other_data", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "keep"
    );
    assert_eq!(
        connection
            .query_row("SELECT version FROM meta WHERE owner='other'", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        7
    );
}

#[test]
fn version_two_cache_retains_native_range_during_pagination_migration() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("migration-v2.sqlite");
    drop(Fetch::open(config(&path)).unwrap());
    let cx = Connection::open(&path).unwrap();
    cx.execute_batch(
        "ALTER TABLE reply DROP COLUMN pagination; UPDATE meta SET version=2 WHERE owner='biomcp-fetch'; \
        INSERT INTO reply(key,source,url,status,content_type,content_range,body,fetched_at,fresh_until) \
        VALUES(zeroblob(32),'pubmed','https://example.test/synthetic',200,'application/json','0-0/1','[]',1,\
        2);",
    )
    .unwrap();
    drop(Fetch::open(config(&path)).unwrap());
    let value: (String, u32, Option<String>) = cx
        .query_row(
            "SELECT content_range,(SELECT version FROM meta WHERE owner='biomcp-fetch'),pagination FROM reply",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(value, ("0-0/1".into(), biomcp_fetch::VERSION, None));
}

#[test]
fn ordinary_reply_write_prunes_expired_rows_and_retains_fresh_rows() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let cx = Connection::open(&path).unwrap();
    cx.execute_batch(
        "INSERT INTO reply(key,source,url,status,body,fetched_at,fresh_until) VALUES \
        (x'01','pubmed','https://example.test/old',200,x'7b7d',0,0),(x'02','pubmed','https://example.test/live',200,x'7b7d',0,9223372036854775807);",
    )
    .unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    let keys: Vec<Vec<u8>> = cx
        .prepare("SELECT key FROM reply ORDER BY key")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&vec![2]));
    assert!(!keys.contains(&vec![1]));
}
