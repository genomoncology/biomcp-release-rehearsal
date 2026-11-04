//! Recordings preserve the public checked transport contract without live providers.
#![expect(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    reason = "Local HTTP fixtures and scoped callers exercise public transport modes"
)]
use biomcp_fetch::{Input, Read};

mod support;
use biomcp_fetch::{
    Arg, Fetch, FetchConfig, KeyUse, Mode, Quota, Rate, RecordError, RequestError, SourceKey,
    SourceSpec,
};
use std::{path::Path, thread, time::Duration};
use support::Answer;
type Server = support::Server<()>;
fn quota(key: &'static str, rate: u16, in_flight: u8) -> &'static Quota {
    Box::leak(Box::new(Quota::new(
        SourceKey::declare(key),
        Rate::per_second(rate),
        None,
        in_flight,
    )))
}
fn spec(server: &Server) -> &'static SourceSpec {
    server.spec(quota("record", 1000, 4), KeyUse::None)
}
fn fetch(mode: Mode) -> Fetch {
    Fetch::open(FetchConfig {
        mode,
        ..FetchConfig::default()
    })
    .unwrap()
}
fn get(fetch: &Fetch, spec: &'static SourceSpec) -> Result<Answer, RequestError> {
    fetch
        .leg(spec, fetch.deadline(Duration::from_secs(10)))
        .call(&spec.get("/answer"), Input::Params(&[]), Read::json())
        .map(|received| received.value)
}
fn file(dir: &Path) -> std::path::PathBuf {
    std::fs::read_dir(dir.join("pubmed"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
}
#[test]
fn repeated_key_consumes_ordered_replies_and_never_opens_replay_http() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    let capture = fetch(Mode::Record {
        dir: dir.path().to_owned(),
    });
    thread::scope(|scope| {
        let worker = scope.spawn(|| [get(&capture, spec), get(&capture, spec)]);
        server.next().ok();
        server.next().respond(200, "", "{broken");
        let replies = worker.join().unwrap();
        assert!(replies[0].as_ref().unwrap().answer);
        assert!(replies[1].is_err());
    });
    drop(capture);
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    assert!(get(&replay, spec).unwrap().answer);
    let restricted = replay.restrict_sources([biomcp_types::Source::Oncokb].into());
    assert!(
        matches!(restricted.leg(spec, replay.deadline(Duration::from_secs(10)))
            .call(&spec.get("/answer"), Input::Params(&[]), Read::<Answer>::json()).map(|received| received.value),
            Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::BadReply)
    );
    assert_eq!(
        get(&replay, spec).unwrap_err(),
        RequestError::Recording(RecordError::Exhausted)
    );
    assert!(server.requests.try_recv().is_err());
    let error = replay
        .leg(spec, replay.deadline(Duration::from_secs(10)))
        .call(
            &spec.get("/answer"),
            Input::Params(&[("other", Arg::Num(1))]),
            Read::<Answer>::json(),
        )
        .unwrap_err();
    assert_eq!(error, RequestError::Recording(RecordError::Missing));
}
#[test]
fn capture_directory_has_one_owner_and_refuses_existing_data() {
    let dir = tempfile::tempdir().unwrap();
    let first = fetch(Mode::Record {
        dir: dir.path().to_owned(),
    });
    assert!(matches!(
        Fetch::open(FetchConfig {
            mode: Mode::Record {
                dir: dir.path().to_owned()
            },
            ..FetchConfig::default()
        }),
        Err(biomcp_fetch::FetchError::Recording(RecordError::Conflict))
    ));
    drop(first);
    std::fs::write(dir.path().join("keep"), "keep").unwrap();
    assert!(matches!(
        Fetch::open(FetchConfig {
            mode: Mode::Record {
                dir: dir.path().to_owned()
            },
            ..FetchConfig::default()
        }),
        Err(biomcp_fetch::FetchError::Recording(RecordError::Conflict))
    ));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("keep")).unwrap(),
        "keep"
    );
}

fn capture(server: &Server, spec: &'static SourceSpec, dir: &Path, replies: &[(u16, &str, &str)]) {
    let recording = fetch(Mode::Record {
        dir: dir.to_owned(),
    });
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            for _ in replies {
                let _ = get(&recording, spec);
            }
        });
        for (status, headers, body) in replies {
            server.next().respond(*status, headers, body);
        }
        worker.join().unwrap();
    });
}
fn change(dir: &Path, update: impl FnOnce(&mut serde_json::Value)) {
    let path = file(dir);
    let mut value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    update(&mut value);
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}
#[test]
fn replay_advances_long_response_and_retry_delays_and_enforces_each_leg_budget() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let original = spec(&server);
    capture(
        &server,
        original,
        dir.path(),
        &[(200, "", r#"{"answer":true}"#)],
    );
    change(dir.path(), |v| {
        let mut success = v["exchanges"][0].clone();
        success["delay_ms"] = 0.into();
        v["exchanges"] = serde_json::json!([
            {"status":503,"content_type":"application/json","delay_ms":120000,"retry_after_ms":240000,"body":{"kind":"text","value":"unavailable"}}, success]);
    });
    let replay = Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: dir.path().to_owned(),
        },
        source_deadlines: [(biomcp_types::Source::PubMed, Duration::from_secs(1000))].into(),
        ..FetchConfig::default()
    })
    .unwrap();
    let clock = replay.deadline(Duration::from_secs(1000));
    let leg = replay.leg(original, clock);
    assert!(
        leg.call(
            &original.get("/answer"),
            Input::Params(&[]),
            Read::<Answer>::json()
        )
        .map(|received| received.value)
        .unwrap()
        .answer
    );
    assert_eq!(leg.remaining(), Duration::from_secs(640));
    assert!(server.requests.try_recv().is_err());
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    let clock = replay.deadline(Duration::from_secs(1000));
    let leg = replay.leg(original, clock);
    assert!(matches!(leg
.call(&original.get("/answer"),
Input::Params(&[]),
Read::<Answer>::json())
.map(|received| received.value),
Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::DeadlineExceeded));
    assert_eq!(leg.remaining(), Duration::ZERO);
    assert!(get(&replay, original).unwrap().answer);
}
#[derive(Debug, serde::Deserialize)]
struct Number {
    value: u32,
}
impl biomcp_fetch::Reply for Number {
    fn check(&self) -> Result<(), biomcp_fetch::BadReply> {
        Ok(())
    }
}
fn capture_numbers(server: &Server, spec: &'static SourceSpec, dir: &Path) {
    let recording = fetch(Mode::Record {
        dir: dir.to_owned(),
    });
    thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    recording
                        .leg(spec, recording.deadline(Duration::from_secs(10)))
                        .call(
                            &spec.get("/answer"),
                            Input::Params(&[]),
                            Read::<Number>::json(),
                        )
                        .map(|received| received.value)
                        .unwrap()
                        .value
                })
            })
            .collect();
        for value in 0..8 {
            server
                .next()
                .respond(200, "", &format!("{{\"value\":{value}}}"));
        }
        let mut numbers: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        numbers.sort();
        assert_eq!(
            numbers,
            (0..8).collect::<Vec<_>>(),
            "Capture must preserve finite numeric request values"
        );
    });
    drop(recording);
}
#[test]
fn concurrent_same_key_reserves_each_exchange_once_and_shares_virtual_quota() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let original = spec(&server);
    capture_numbers(&server, original, dir.path());
    change(dir.path(), |v| {
        for exchange in v["exchanges"].as_array_mut().unwrap() {
            exchange["delay_ms"] = 0.into();
        }
    });
    let quota = quota("virtual-shared", 1, 2);
    let slow = server.spec(quota, KeyUse::None);
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    let clock = replay.deadline(Duration::from_secs(20));
    let mut numbers = thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let leg = replay.leg(slow, clock);
                    let value = leg
                        .call(
                            &slow.get("/answer"),
                            Input::Params(&[]),
                            Read::<Number>::json(),
                        )
                        .map(|received| received.value)
                        .unwrap()
                        .value;
                    (value, leg.remaining())
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>()
    });
    let mut budgets: Vec<_> = numbers.iter().map(|(_, remaining)| *remaining).collect();
    budgets.sort();
    assert_eq!(
        budgets,
        (3..=10).map(Duration::from_secs).collect::<Vec<_>>()
    );
    numbers.sort();
    assert_eq!(
        numbers.iter().map(|(value, _)| *value).collect::<Vec<_>>(),
        (0..8).collect::<Vec<_>>()
    );
    assert_eq!(
        get(&replay, slow).unwrap_err(),
        RequestError::Recording(RecordError::Exhausted)
    );
    assert!(server.requests.try_recv().is_err());
}
#[test]
fn invalid_recordings_are_local_bounded_errors_with_no_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    capture(
        &server,
        spec,
        dir.path(),
        &[(200, "", r#"{"answer":true}"#)],
    );
    let original = std::fs::read(file(dir.path())).unwrap();
    for (field, value) in [
        ("status", serde_json::json!(999)),
        ("delay_ms", serde_json::json!(-1)),
        ("retry_after_ms", serde_json::json!(18446744073709551616.0)),
    ] {
        std::fs::write(file(dir.path()), &original).unwrap();
        change(dir.path(), |v| v["exchanges"][0][field] = value);
        let replay = fetch(Mode::Replay {
            dir: dir.path().to_owned(),
        });
        assert_eq!(
            get(&replay, spec).unwrap_err(),
            RequestError::Recording(RecordError::Malformed)
        );
    }
    std::fs::write(file(dir.path()), &original).unwrap();
    change(dir.path(), |v| v["url"] = "wrong".into());
    assert_eq!(
        get(
            &fetch(Mode::Replay {
                dir: dir.path().to_owned()
            }),
            spec
        )
        .unwrap_err(),
        RequestError::Recording(RecordError::Malformed)
    );
    std::fs::write(file(dir.path()), vec![b' '; 16 * 1024 * 1024 + 1]).unwrap();
    assert_eq!(
        get(
            &fetch(Mode::Replay {
                dir: dir.path().to_owned()
            }),
            spec
        )
        .unwrap_err(),
        RequestError::Recording(RecordError::Limit)
    );
    assert!(server.requests.try_recv().is_err());
}
#[test]
fn capture_keeps_retry_status_headers_and_rejected_json_without_credentials() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let credential = biomcp_fetch::Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: biomcp_fetch::SecretName::NcbiApiKey,
        place: biomcp_fetch::KeyPlace::Query("api_key"),
    };
    let spec = server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("capture-key"),
            Rate::per_second(1000),
            None,
            1,
        ))),
        KeyUse::Required(credential),
    );
    let mut config = FetchConfig {
        mode: Mode::Record {
            dir: dir.path().to_owned(),
        },
        ..FetchConfig::default()
    };
    config
        .secrets
        .insert(credential.secret, biomcp_fetch::Secret::new("private-key"));
    let recording = Fetch::open(config).unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| get(&recording, spec));
        server.next().respond(429, "Retry-After: 0\r\n", "limited");
        server.next().respond(200, "", r#"{"answer":false}"#);
        assert!(
            matches!(worker.join().unwrap(), Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::BadReply)
        );
    });
    drop(recording);
    let text = std::fs::read_to_string(file(dir.path())).unwrap();
    assert!(!text.contains("private-key") && !text.contains("api_key"));
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["exchanges"][0]["retry_after_ms"], 0);
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    assert!(
        matches!(get(&replay, spec), Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::NoKey)
    );
    // Missing credentials win even over a now malformed recording.
    std::fs::write(file(dir.path()), "bad").unwrap();
    assert!(
        matches!(get(&replay, spec), Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::NoKey)
    );
}

#[test]
fn non_utf8_capture_is_a_typed_refusal_and_never_publishes_a_partial_file() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    let recording = fetch(Mode::Record {
        dir: dir.path().to_owned(),
    });
    thread::scope(|scope| {
        let worker = scope.spawn(|| get(&recording, spec));
        let mut response = b"HTTP/1.1 200 Stub\r\nContent-Type: application/json\r\nContent-Length: 1\r\nConnection: close\r\n\r\n".to_vec();
        response.push(255);
        server.next().bytes(response);
        assert_eq!(
            worker.join().unwrap().unwrap_err(),
            RequestError::Recording(RecordError::NonUtf8)
        );
    });
    assert!(!dir.path().join("pubmed").exists());
}

#[test]
fn independent_virtual_leg_budgets_reject_foreign_clock_domains_before_recording_access() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    capture(
        &server,
        spec,
        dir.path(),
        &[
            (200, "", r#"{"answer":true}"#),
            (200, "", r#"{"answer":true}"#),
        ],
    );
    change(dir.path(), |v| {
        v["exchanges"][0]["delay_ms"] = 6000.into();
        v["exchanges"][1]["delay_ms"] = 0.into();
    });
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    let other = server.spec(quota("independent-clock", 1000, 1), KeyUse::None);
    let budget = replay.deadline(Duration::from_secs(10));
    let first = replay.leg(spec, budget);
    let second = replay.leg(other, budget);
    assert!(
        first
            .call(
                &spec.get("/answer"),
                Input::Params(&[]),
                Read::<Answer>::json()
            )
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    assert_eq!(first.remaining(), Duration::from_secs(4));
    assert!(
        second
            .call(
                &other.get("/answer"),
                Input::Params(&[]),
                Read::<Answer>::json()
            )
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    assert_eq!(second.remaining(), Duration::from_secs(10));
    std::fs::write(file(dir.path()), "broken-recording").unwrap();
    let live = fetch(Mode::default());
    let foreign = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    for wrong in [
        live.deadline(Duration::from_secs(10)),
        foreign.deadline(Duration::from_secs(10)),
    ] {
        assert!(matches!(replay.leg(spec,
wrong).call(&spec.get("/answer"), Input::Params(&[]), Read::<Answer>::json()).map(|received| received.value),
 Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::Rejected));
    }
    assert!(matches!(live.leg(spec,budget)
.call(&spec.get("/answer"),
Input::Params(&[]),
Read::<Answer>::json())
.map(|received| received.value),
Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::Rejected));
    assert!(server.requests.try_recv().is_err());
}

#[path = "record/bytes.rs"]
mod bytes;

#[test]
fn binary_sidecars_are_ordered_and_confined() {
    bytes::ordered_binary_sidecars_round_trip_and_refuse_hostile_paths();
}

#[test]
fn cut_binary_body_never_returns_partial_bytes() {
    bytes::cut_binary_body_is_refused_before_returning_bytes();
}

#[test]
fn bounded_json_overflow_preserves_existing_reader_errors() {
    bytes::only_bounded_json_reports_typed_overflow();
}

#[test]
fn head_metadata_capture_never_reads_the_object_or_collides_with_get() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    let capture = fetch(Mode::Record {
        dir: dir.path().to_owned(),
    });
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let leg = capture.leg(spec, capture.deadline(Duration::from_secs(10)));
            leg.probe(&spec.head("/answer"), biomcp_fetch::Input::Params(&[]))
                .unwrap();
            get(&capture, spec).unwrap();
        });
        let request = server.next();
        assert!(request.text.starts_with("HEAD /answer "));
        assert!(request.body.is_empty());
        request.raw("HTTP/1.1 200 OK\r\nContent-Length: 999999999\r\nConnection: close\r\n\r\n");
        server.next().ok();
        worker.join().unwrap();
    });
    drop(capture);
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    assert!(get(&replay, spec).unwrap().answer);
    replay
        .leg(spec, replay.deadline(Duration::from_secs(10)))
        .probe(&spec.head("/answer"), biomcp_fetch::Input::Params(&[]))
        .unwrap();
    assert!(server.requests.try_recv().is_err());
    let path = std::fs::read_dir(dir.path().join("pubmed"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            let value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            value["method"] == "HEAD"
        })
        .unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for mutation in ["GET", "body"] {
        let mut value = original.clone();
        if mutation == "GET" {
            value["method"] = "GET".into();
        } else {
            value["exchanges"][0]["body"]["value"] = "unexpected object bytes".into();
        }
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let replay = fetch(Mode::Replay {
            dir: dir.path().to_owned(),
        });
        assert_eq!(
            replay
                .leg(spec, replay.deadline(Duration::from_secs(10)))
                .probe(&spec.head("/answer"), biomcp_fetch::Input::Params(&[]))
                .unwrap_err(),
            RequestError::Recording(RecordError::Malformed)
        );
    }
}

#[path = "record/metadata.rs"]
mod metadata;

#[path = "record/errors.rs"]
mod errors;
