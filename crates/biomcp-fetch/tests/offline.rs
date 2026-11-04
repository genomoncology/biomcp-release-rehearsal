//! Checked cache isolation and offline behavior through the public request API.
#![expect(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    reason = "Controlled local peers seed explicit isolated caches"
)]
use biomcp_fetch::{Input, Read};

mod support;
use biomcp_fetch::{
    Fetch, FetchConfig, Freshness, KeyUse, Mode, Quota, Rate, RecordError, ReplayCache,
    RequestError, SourceKey,
};
use biomcp_types::Reason;
use std::{path::Path, thread, time::Duration};
use support::Answer;
type Server = support::Server<()>;
fn config(cache: &Path, mode: Mode) -> FetchConfig {
    FetchConfig {
        cache: Some(cache.to_owned()),
        mode,
        ..FetchConfig::default()
    }
}
fn spec(server: &Server, key: &'static str) -> &'static biomcp_fetch::SourceSpec {
    server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare(key),
            Rate::per_second(1000),
            None,
            1,
        ))),
        KeyUse::None,
    )
}
fn expire(path: &Path) {
    rusqlite::Connection::open(path)
        .unwrap()
        .execute("UPDATE reply SET fresh_until=0", [])
        .unwrap();
}
fn rows(path: &Path) -> u32 {
    rusqlite::Connection::open(path)
        .unwrap()
        .query_row("SELECT count(*) FROM reply", [], |r| r.get(0))
        .unwrap()
}
#[test]
fn offline_rechecks_fresh_stale_negative_expired_and_corrupt_entries_without_http() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, "offline");
    let ep = spec.get("/answer").fresh(Freshness::Hours(1));
    let live = Fetch::open(config(&cache, Mode::Live { fresh: true })).unwrap();
    let offline = Fetch::open(config(&cache, Mode::Offline)).unwrap();
    let get = |fetch: &Fetch| {
        fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(&ep, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
    };
    assert!(matches!(get(&offline), Err(RequestError::Source(e)) if e.reason == Reason::Offline));
    thread::scope(|scope| {
        let worker = scope.spawn(|| get(&live));
        server.next().ok();
        worker.join().unwrap().unwrap();
    });
    assert!(get(&offline).unwrap().answer);
    expire(&cache);
    let leg = offline.leg(spec, offline.deadline(Duration::from_secs(10)));
    assert!(
        leg.call(&ep, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    let report = leg.finish();
    assert!(report.stale);
    assert!(report.failures.is_empty());
    thread::scope(|scope| {
        let worker = scope.spawn(|| get(&live));
        server.next().respond(404, "", "missing");
        assert!(worker.join().unwrap().is_err());
    });
    assert!(
        offline
            .leg(spec, offline.deadline(Duration::from_secs(10)))
            .call(&ep, Input::Params(&[]), Read::<Answer>::json())
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
    expire(&cache);
    assert!(matches!(get(&offline), Err(RequestError::Source(e)) if e.reason == Reason::Offline));
    // A positive row invalidated by the caller's check is evicted without sending.
    rusqlite::Connection::open(&cache)
        .unwrap()
        .execute(
            "UPDATE reply SET status=200, body=?1, fresh_until=9999999999",
            [b"{\"answer\":false}".as_slice()],
        )
        .unwrap();
    assert!(matches!(get(&offline), Err(RequestError::Source(e)) if e.reason == Reason::Offline));
    assert_eq!(rows(&cache), 0);
    assert!(server.requests.try_recv().is_err());
}
#[test]
fn record_bypasses_warm_cache_and_replay_ignores_user_cache_and_writes_only_owned_case_cache() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.sqlite");
    let recording = dir.path().join("recording");
    let server = Server::new();
    let spec = spec(&server, "isolated-replay");
    let ep = spec.get("/answer").fresh(Freshness::Hours(1));
    let get = |fetch: &Fetch| {
        fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(&ep, Input::Params(&[]), Read::<Answer>::json())
    };
    let live = Fetch::open(config(&cache, Mode::default())).unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| get(&live));
        server.next().ok();
        worker.join().unwrap().unwrap();
    });
    let capture = Fetch::open(config(
        &cache,
        Mode::Record {
            dir: recording.clone(),
        },
    ))
    .unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| [get(&capture), get(&capture)]);
        server.next().respond(200, "", "{broken");
        server.next().ok();
        let replies = worker.join().unwrap();
        assert!(replies[0].is_err());
        assert!(replies[1].as_ref().unwrap().value.answer);
    });
    drop(capture);
    let connection = rusqlite::Connection::open(&cache).unwrap();
    let before = cached_body(&connection);
    let case = ReplayCache::temporary().unwrap();
    let replay = Fetch::open(FetchConfig {
        replay_cache: Some(case.clone()),
        ..config(
            &cache,
            Mode::Replay {
                dir: recording.clone(),
            },
        )
    })
    .unwrap();
    assert!(get(&replay).is_err());
    assert!(get(&replay).unwrap().value.answer);
    assert!(get(&replay).unwrap().value.answer);
    assert_eq!(
        replay
            .leg(spec, replay.deadline(Duration::from_secs(10)))
            .fresh(true)
            .call(&ep, Input::Params(&[]), Read::<Answer>::json())
            .unwrap_err(),
        RequestError::Recording(RecordError::Exhausted)
    );
    let offline = Fetch::open(FetchConfig {
        replay_cache: Some(case),
        ..config(&dir.path().join("must-not-open"), Mode::Offline)
    })
    .unwrap();
    drop(replay);
    assert!(get(&offline).unwrap().value.answer);
    assert!(!dir.path().join("must-not-open").exists());
    let after = cached_body(&connection);
    assert_eq!(before, after);
    let missing = Fetch::open(config(
        &cache,
        Mode::Replay {
            dir: dir.path().join("missing"),
        },
    ))
    .unwrap();
    assert_eq!(
        get(&missing).unwrap_err(),
        RequestError::Recording(RecordError::Missing)
    );
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn replay_does_not_open_an_invalid_ordinary_cache() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("invalid.sqlite");
    std::fs::write(&cache, b"keep").unwrap();
    let recording = dir.path().join("recording");
    let server = Server::new();
    let spec = spec(&server, "invalid-ordinary");
    let ep = spec.get("/answer").fresh(Freshness::Hours(1));
    let capture = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: recording.clone(),
        },
        ..FetchConfig::default()
    })
    .unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            capture
                .leg(spec, capture.deadline(Duration::from_secs(10)))
                .call(&ep, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value)
        });
        server.next().ok();
        worker.join().unwrap().unwrap();
    });
    drop(capture);
    let replay = Fetch::open(config(&cache, Mode::Replay { dir: recording })).unwrap();
    assert!(
        replay
            .leg(spec, replay.deadline(Duration::from_secs(10)))
            .call(&ep, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    assert_eq!(std::fs::read(cache).unwrap(), b"keep");
    assert!(server.requests.try_recv().is_err());
}

fn cached_body(connection: &rusqlite::Connection) -> Vec<u8> {
    connection
        .query_row("SELECT body FROM reply", [], |r| r.get(0))
        .unwrap()
}
