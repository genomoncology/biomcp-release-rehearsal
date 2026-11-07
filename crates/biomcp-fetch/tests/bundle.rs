//! Bundle contracts through the public transport boundary.
#![expect(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    reason = "Fixture failures fail the contract"
)]
use biomcp_fetch::Input;

use biomcp_test_support::environment as child_environment;
mod support;
use biomcp_fetch::{
    Bundle, Fetch, FetchConfig, Find, Freshness, KeyKind, KeyUse, Keys, Quota, Rate, Row,
    RowFormat, SourceKey,
};
use serde::Deserialize;
use std::{marker::PhantomData, time::Duration};
#[derive(Debug, Deserialize)]
struct Gene {
    symbol: String,
    band: String,
    name: String,
}
impl Row for Gene {
    fn keys(&self) -> Keys {
        Keys {
            exact: vec![
                (KeyKind::Symbol, self.symbol.clone().into()),
                (KeyKind::Band, self.band.clone().into()),
            ],
            words: vec![self.name.clone().into()],
        }
    }
}
const QUOTA: Quota = Quota::new(
    SourceKey::declare("bundle-contract"),
    Rate::per_second(1000),
    None,
    4,
);
fn declared(server: &support::Server) -> Bundle<Gene> {
    Bundle {
        spec: server.spec(&QUOTA, KeyUse::None),
        name: "genes",
        path: "/genes",
        format: RowFormat::Tsv,
        fresh: Freshness::Days(1),
        row: PhantomData,
    }
}
const TSV: &str = "symbol\tband\tname\nBRAF\t9p21.1\t\"B raf kinase\"\n";
fn cached(path: &std::path::Path) -> Fetch {
    Fetch::open(FetchConfig {
        cache: Some(path.to_owned()),
        ..Default::default()
    })
    .unwrap()
}
fn sync_reply(
    fetch: &Fetch,
    bundle: &Bundle<Gene>,
    server: &support::Server,
    status: u16,
    headers: &str,
    body: &str,
) -> Result<biomcp_fetch::BundleInfo, biomcp_fetch::RequestError> {
    std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync(bundle));
        server.next().respond(status, headers, body);
        call.join().unwrap()
    })
}
#[test]
fn source_failure_retains_stale_rows_but_local_errors_do_not() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let f = cached(&path);
    sync_reply(&f, &b, &server, 200, "", TSV).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute("UPDATE bundle SET fresh_until=0", []).unwrap();
    let leg = f.leg(b.spec, f.deadline(Duration::from_secs(5)));
    std::thread::scope(|scope| {
        let call = scope.spawn(|| leg.rows(&b, Find::Exact(KeyKind::Symbol, "BRAF")));
        for _ in 0..3 {
            server.next().respond(500, "", "");
        }
        assert_eq!(call.join().unwrap().unwrap().len(), 1);
    });
    let report = leg.finish();
    assert!(report.stale);
    assert_eq!(report.failures[0].status, Some(500));
    db.execute_batch("DROP TABLE bundle_key").unwrap();
    assert!(matches!(
        f.leg(b.spec, f.deadline(Duration::from_secs(1)))
            .rows(&b, Find::Exact(KeyKind::Symbol, "BRAF")),
        Err(biomcp_fetch::RequestError::Cache(_))
    ));
}
#[test]
fn recording_replay_replaces_warm_case_rows_once_and_ignores_user_cache() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let capture = dir.path().join("recordings");
    let path = dir.path().join("live.sqlite");
    let initial = cached(&path);
    sync_reply(&initial, &b, &server, 200, "", TSV).unwrap();
    let record = Fetch::open(FetchConfig {
        cache: Some(path.clone()),
        mode: biomcp_fetch::Mode::Record {
            dir: capture.clone(),
        },
        ..Default::default()
    })
    .unwrap();
    std::thread::scope(|scope| {
        let call = scope.spawn(|| {
            let leg = record.leg(b.spec, record.deadline(Duration::from_secs(30)));
            leg.rows(&b, Find::Exact(KeyKind::Symbol, "NEW"))
        });
        let request = server.next();
        assert!(!request.text.to_ascii_lowercase().contains("if-"));
        request.respond(200, "", "symbol\tband\tname\nNEW\t9p\t\"tab\\line\"\n");
        assert_eq!(call.join().unwrap().unwrap().len(), 1);
    });
    drop(record);
    let case = biomcp_fetch::ReplayCache::temporary().unwrap();
    let recording = recording_path(&capture);
    for round in 0..2 {
        if round == 1 {
            replace_recorded_body(&recording, TSV);
        }
        let replay = Fetch::open(FetchConfig {
            cache: Some(dir.path().join("must-not-exist.sqlite")),
            mode: biomcp_fetch::Mode::Replay {
                dir: capture.clone(),
            },
            replay_cache: Some(case.clone()),
            ..Default::default()
        })
        .unwrap();
        let leg = replay.leg(b.spec, replay.deadline(Duration::from_secs(5)));
        let (present, absent) = if round == 0 {
            ("NEW", "BRAF")
        } else {
            ("BRAF", "NEW")
        };
        assert_eq!(
            leg.rows(&b, Find::Exact(KeyKind::Symbol, present))
                .unwrap()
                .len(),
            1
        );
        assert!(
            leg.rows(&b, Find::Exact(KeyKind::Symbol, absent))
                .unwrap()
                .is_empty()
        );
        assert!(!dir.path().join("must-not-exist.sqlite").exists());
    }
    let offline = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Offline,
        replay_cache: Some(case),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        offline
            .leg(b.spec, offline.deadline(Duration::from_secs(1)))
            .rows(&b, Find::Prefix(KeyKind::Symbol, ""))
            .unwrap()
            .len(),
        1
    );
    assert!(server.requests.try_recv().is_err());
}
struct Child {
    process: std::process::Child,
    _environment: tempfile::TempDir,
}
impl Child {
    fn start(path: &std::path::Path, base: &str, expected: &str) -> Self {
        let environment = child_environment::new().unwrap();
        let mut command =
            child_environment::command(&std::env::current_exe().unwrap(), environment.path(), &[]);
        command
            .args(["--exact", "bundle_sync_child", "--nocapture"])
            .env("BUNDLE_PATH", path)
            .env("BUNDLE_BASE", base)
            .env("BUNDLE_EXPECTED", expected)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        Self {
            process: command.spawn().unwrap(),
            _environment: environment,
        }
    }
    fn finish(&mut self) {
        let end = std::time::Instant::now() + Duration::from_secs(15);
        loop {
            if let Some(status) = self.process.try_wait().unwrap() {
                let mut output = String::new();
                use std::io::Read;
                self.process
                    .stdout
                    .take()
                    .unwrap()
                    .read_to_string(&mut output)
                    .unwrap();
                self.process
                    .stderr
                    .take()
                    .unwrap()
                    .read_to_string(&mut output)
                    .unwrap();
                assert!(status.success(), "{output}");
                break;
            }
            assert!(std::time::Instant::now() < end, "bundle child stalled");
            std::thread::park_timeout(Duration::from_millis(1));
        }
    }
}
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}
#[test]
fn bundle_sync_child() {
    let Ok(path) = std::env::var("BUNDLE_PATH") else {
        return;
    };
    let base = Box::leak(std::env::var("BUNDLE_BASE").unwrap().into_boxed_str());
    let spec = Box::leak(Box::new(biomcp_fetch::SourceSpec {
        destinations: biomcp_fetch::Destinations::Primary,
        user_agent: None,
        terms: biomcp_fetch::Terms {
            name: "Synthetic test data",
            url: "https://example.test/terms",
            usage: biomcp_types::Use::Open,
        },
        base,
        source: biomcp_types::Source::PubMed,
        key: KeyUse::None,
        quota: &QUOTA,
        deadline: Duration::from_secs(10),
    }));
    let b = Bundle::<Gene> {
        spec,
        name: "genes",
        path: "/genes",
        format: RowFormat::Tsv,
        fresh: Freshness::Days(1),
        row: PhantomData,
    };
    let f = cached(std::path::Path::new(&path));
    let result = f.sync(&b);
    if std::env::var("BUNDLE_EXPECTED").unwrap() == "syncing" {
        assert!(
            matches!(result,Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::Syncing)
        );
    } else {
        assert_eq!(result.unwrap().rows, 1);
    }
}
#[test]
fn independent_holders_contend_and_takeover_fences_the_old_publisher() {
    let server = support::Server::<()>::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let _ = cached(&path);
    let mut old = Child::start(&path, server.base, "syncing");
    let old_request = server.next();
    let mut contender = Child::start(&path, server.base, "syncing");
    contender.finish();
    assert!(server.requests.try_recv().is_err());
    let db = rusqlite::Connection::open(&path).unwrap();
    let old_path: String = db
        .query_row("SELECT part FROM bundle_sync", [], |r| r.get(0))
        .unwrap();
    db.execute("UPDATE bundle_sync SET lease_until=0", [])
        .unwrap();
    let mut successor = Child::start(&path, server.base, "ok");
    let next_request = server.next();
    let successor_path: String = db
        .query_row("SELECT part FROM bundle_sync", [], |r| r.get(0))
        .unwrap();
    assert_ne!(old_path, successor_path);
    old_request.respond(200, "ETag: \"old\"\r\n", TSV);
    old.finish();
    assert!(std::path::Path::new(&successor_path).exists());
    assert_eq!(
        db.query_row(
            "SELECT count(*) FROM bundle_sync WHERE holder IS NOT NULL",
            [],
            |r| r.get::<_, u32>(0)
        )
        .unwrap(),
        1
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM bundle_row", [], |r| r
            .get::<_, u32>(0))
            .unwrap(),
        0
    );
    next_request.respond(200, "ETag: \"new\"\r\n", TSV);
    successor.finish();
    assert_eq!(
        db.query_row("SELECT etag FROM bundle", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "\"new\""
    );
    assert!(!std::path::Path::new(&old_path).exists());
}
#[test]
fn bundle_source_and_storage_boundaries_are_checked_before_requests() {
    let server = support::Server::<()>::new();
    let b = declared(&server);
    let foreign = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let f = cached(&path);
    assert!(matches!(f.leg(foreign.spec,
f.deadline(Duration::from_secs(1))).rows(&b,
Find::Words("x")),
Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::Rejected));
    assert!(matches!(f.leg(b.spec,
f.deadline(Duration::ZERO)).rows(&b,
Find::Words("x")),
Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::DeadlineExceeded));
    let uncached = Fetch::open(FetchConfig::default()).unwrap();
    assert!(matches!(
        uncached.sync(&b),
        Err(biomcp_fetch::RequestError::Cache(_))
    ));
    let offline = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })
    .unwrap();
    assert!(
        matches!(offline.sync(&b),Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::Offline)
    );
    assert!(matches!(offline.leg(b.spec,
offline.deadline(Duration::from_secs(1))).rows(&b,
Find::Words("x")),
Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::Offline));
    assert!(server.requests.try_recv().is_err());
}
#[test]
#[ignore = "Release proof: body larger than the prior recording-file cap"]
fn release_large_bundle_capture_round_trips() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let capture = dir.path().join("recordings");
    let body = format!(
        "symbol\tband\tname\nBRAF\t9p\t{}\n",
        "x".repeat(17 * 1024 * 1024)
    );
    let record = Fetch::open(FetchConfig {
        cache: Some(dir.path().join("cache.sqlite")),
        mode: biomcp_fetch::Mode::Record {
            dir: capture.clone(),
        },
        max_body: 64,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        sync_reply(&record, &b, &server, 200, "", &body)
            .unwrap()
            .bytes,
        body.len() as u64
    );
    drop(record);
    let replay = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Replay { dir: capture },
        max_body: 64,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(replay.sync(&b).unwrap().bytes, body.len() as u64);
    // A larger ordinary response limit does not grant the bundle recording cap.
    let ordinary = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Replay {
            dir: dir.path().join("recordings"),
        },
        max_body: biomcp_fetch::MAX_BUNDLE_BYTES,
        ..Default::default()
    })
    .unwrap();
    ordinary.sync(&b).unwrap();
    assert!(matches!(
        ordinary
            .leg(b.spec, ordinary.deadline(Duration::from_secs(5)))
            .call(
                &b.spec.get(b.path),
                Input::Params(&[]),
                biomcp_fetch::Read::<support::Answer>::json()
            )
            .map(|received| received.value),
        Err(biomcp_fetch::RequestError::Recording(
            biomcp_fetch::RecordError::Limit
        ))
    ));
}

fn recording_path(dir: &std::path::Path) -> std::path::PathBuf {
    let source = std::fs::read_dir(dir)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::read_dir(source)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
}
fn replace_recorded_body(path: &std::path::Path, body: &str) {
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    *value.pointer_mut("/exchanges/0/body/value").unwrap() = body.into();
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
}

#[path = "bundle/download.rs"]
mod download;
#[path = "bundle/rows.rs"]
mod rows;

#[path = "bundle/native.rs"]
mod native;

#[path = "bundle/rejections.rs"]
mod rejections;

#[path = "bundle/resume.rs"]
mod resume;
