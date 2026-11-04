//! Declared POST bodies share the checked transport and modes.
#![expect(
    clippy::disallowed_methods,
    reason = "Local HTTP fixtures and scoped public API callers"
)]
use biomcp_fetch::Read;

mod support;
use biomcp_fetch::{Arg, Fetch, FetchConfig, KeyUse, Quota, Rate, SourceKey};
use std::{thread, time::Duration};
use support::Answer;
type Server = support::Server<()>;
fn spec(server: &Server) -> &'static biomcp_fetch::SourceSpec {
    server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("post"),
            Rate::per_second(1000),
            None,
            4,
        ))),
        KeyUse::None,
    )
}
#[test]
fn declared_form_preserves_order_duplicates_and_encodes_sorted_ids() {
    let server = Server::new();
    let spec = spec(&server);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let ids = ["Z".into(), "A".into()];
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(10)))
                .call(
                    &spec.post_form("/answer"),
                    biomcp_fetch::Input::Params(&[
                        ("q", Arg::Text("a +é")),
                        ("q", Arg::Num(2)),
                        ("ids", Arg::Ids(&ids)),
                    ]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let request = server.next();
        assert!(request.text.starts_with("POST /answer HTTP/1.1\r\n"));
        assert!(
            request
                .text
                .to_ascii_lowercase()
                .contains("content-type: application/x-www-form-urlencoded\r\n")
        );
        assert_eq!(request.body, b"q=a+%2B%C3%A9&q=2&ids=A%2CZ");
        request.ok();
        assert!(worker.join().unwrap().unwrap().answer);
    });
}

#[expect(
    clippy::disallowed_types,
    reason = "Count serialization only within this request-body fixture"
)]
struct Once<'a>(&'a std::sync::atomic::AtomicUsize);
impl serde::Serialize for Once<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        assert_eq!(
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            0,
            "A request body must serialize only once before retry"
        );
        serde::Serialize::serialize(&serde_json::json!({"query":"a +é"}), serializer)
    }
}
#[test]
#[expect(
    clippy::disallowed_types,
    reason = "Observe owned test workers without leaking shared state across cases"
)]
fn json_serializes_once_and_only_declared_idempotent_posts_retry() {
    let server = Server::new();
    let spec = spec(&server);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let calls = std::sync::atomic::AtomicUsize::new(0);
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(10)))
                .call(
                    &spec.post_json("/answer").idempotent(),
                    biomcp_fetch::Input::json(&[], &Once(&calls)),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        for status in [503, 429, 200] {
            let request = server.next();
            assert!(
                request
                    .text
                    .to_ascii_lowercase()
                    .contains("content-type: application/json\r\n")
            );
            assert_eq!(request.body, "{\"query\":\"a +é\"}".as_bytes());
            request.respond(status, "Retry-After: 0\r\n", r#"{"answer":true}"#);
        }
        assert!(worker.join().unwrap().unwrap().answer);
    });
    for status in [429, 503] {
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(10)))
                    .call(
                        &spec.post_json("/answer"),
                        biomcp_fetch::Input::json(&[], &true),
                        Read::<Answer>::json(),
                    )
                    .map(|received| received.value)
            });
            server
                .next()
                .respond(status, "Retry-After: 99999\r\n", "bad");
            let error = worker.join().unwrap().unwrap_err();
            assert!(
                matches!(error, biomcp_fetch::RequestError::Source(e) if e.status == Some(status) && e.retry_after == Some(Duration::from_secs(99999)))
            );
            assert!(server.requests.try_recv().is_err());
        });
    }
}
struct Broken;
impl serde::Serialize for Broken {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("private-serializer-text"))
    }
}
#[test]
fn preparation_errors_precede_locked_cache_and_malformed_replay_without_echo() {
    use biomcp_fetch::{Freshness, InputError, Mode, RequestError};
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    for mode in [
        Mode::default(),
        Mode::Replay {
            dir: dir.path().join("missing"),
        },
        Mode::Record {
            dir: dir.path().join("capture"),
        },
    ] {
        let path = dir.path().join("cache.sqlite");
        let fetch = Fetch::open(FetchConfig {
            mode,
            cache: Some(path.clone()),
            max_body: 32,
            ..FetchConfig::default()
        })
        .unwrap();
        let database = rusqlite::Connection::open(path).unwrap();
        database.execute_batch("BEGIN EXCLUSIVE").unwrap();
        let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
        let endpoint = spec.post_json("/answer").fresh(Freshness::Days(1));
        for (error, expected) in [
            (
                leg.call(
                    &endpoint,
                    biomcp_fetch::Input::json(&[], &Broken),
                    Read::<Answer>::json(),
                )
                .unwrap_err(),
                InputError::Serialization,
            ),
            (
                leg.call(
                    &endpoint,
                    biomcp_fetch::Input::json(&[], &"x".repeat(33)),
                    Read::<Answer>::json(),
                )
                .unwrap_err(),
                InputError::BodyLimit,
            ),
            (
                leg.call(
                    &endpoint,
                    biomcp_fetch::Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .unwrap_err(),
                InputError::Method,
            ),
            (
                leg.call(
                    &spec.get("/answer"),
                    biomcp_fetch::Input::json(&[], &Broken),
                    Read::<Answer>::json(),
                )
                .unwrap_err(),
                InputError::Method,
            ),
            (
                leg.call(
                    &spec.post_form("/answer"),
                    biomcp_fetch::Input::Params(&[("q", Arg::Text(&"é".repeat(20)))]),
                    Read::<Answer>::json(),
                )
                .unwrap_err(),
                InputError::BodyLimit,
            ),
        ] {
            assert_eq!(error, RequestError::Input(expected));
            assert!(!format!("{error:?} {error}").contains("private-serializer-text"));
        }
        database.execute_batch("ROLLBACK").unwrap();
        assert!(server.requests.try_recv().is_err());
        assert!(!dir.path().join("capture/pubmed").exists());
    }
}

fn invoke(
    fetch: &Fetch,
    spec: &'static biomcp_fetch::SourceSpec,
    kind: u8,
) -> Result<Answer, biomcp_fetch::RequestError> {
    use biomcp_fetch::Freshness;
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    match kind {
        0 => leg
            .call(
                &spec.get("/answer").fresh(Freshness::Days(1)),
                biomcp_fetch::Input::Params(&[("q", Arg::Num(1))]),
                Read::json(),
            )
            .map(|received| received.value),
        1 => leg
            .call(
                &spec.post_form("/answer").fresh(Freshness::Days(1)),
                biomcp_fetch::Input::Params(&[("q", Arg::Num(1))]),
                Read::json(),
            )
            .map(|received| received.value),
        2 => leg
            .call(
                &spec.post_json("/answer").fresh(Freshness::Days(1)),
                biomcp_fetch::Input::json(&[], &serde_json::json!({"q":1})),
                Read::json(),
            )
            .map(|received| received.value),
        _ => leg
            .call(
                &spec.post_json("/answer").fresh(Freshness::Days(1)),
                biomcp_fetch::Input::json(&[], &serde_json::json!({"q":2})),
                Read::json(),
            )
            .map(|received| received.value),
    }
}
#[test]
fn cache_capture_replay_and_offline_distinguish_method_encoding_and_body() {
    use biomcp_fetch::{Mode, RecordError, ReplayCache, RequestError};
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    let cache = dir.path().join("cache.sqlite");
    let live = Fetch::open(FetchConfig {
        cache: Some(cache.clone()),
        ..FetchConfig::default()
    })
    .unwrap();
    for kind in 0..4 {
        thread::scope(|scope| {
            let worker = scope.spawn(|| invoke(&live, spec, kind));
            server.next().ok();
            assert!(worker.join().unwrap().unwrap().answer);
        });
        assert!(invoke(&live, spec, kind).unwrap().answer);
        assert!(server.requests.try_recv().is_err());
    }
    let capture = dir.path().join("capture");
    let recording = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: capture.clone(),
        },
        cache: Some(cache.clone()),
        ..FetchConfig::default()
    })
    .unwrap();
    for kind in 0..4 {
        thread::scope(|scope| {
            let worker = scope.spawn(|| invoke(&recording, spec, kind));
            // Record bypasses all the already warm cache entries.
            server.next().ok();
            assert!(worker.join().unwrap().unwrap().answer);
        });
    }
    drop(recording);
    assert_eq!(
        std::fs::read_dir(capture.join("pubmed")).unwrap().count(),
        4
    );
    let case_cache = ReplayCache::temporary().unwrap();
    let replay = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: capture },
        cache: Some(cache),
        replay_cache: Some(case_cache.clone()),
        ..FetchConfig::default()
    })
    .unwrap();
    for kind in 0..4 {
        assert!(invoke(&replay, spec, kind).unwrap().answer);
        assert!(invoke(&replay, spec, kind).unwrap().answer);
    }
    let leg = replay.leg(spec, replay.deadline(Duration::from_secs(10)));
    assert_eq!(
        leg.call(
            &spec.post_json("/answer"),
            biomcp_fetch::Input::json(&[], &serde_json::json!({"q":3})),
            Read::<Answer>::json()
        )
        .unwrap_err(),
        RequestError::Recording(RecordError::Missing)
    );
    let offline = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        replay_cache: Some(case_cache),
        ..FetchConfig::default()
    })
    .unwrap();
    for kind in 0..4 {
        assert!(invoke(&offline, spec, kind).unwrap().answer);
    }
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn credentials_are_inserted_only_on_wire_and_capture_metadata_is_checked_before_consumption() {
    use biomcp_fetch::{Credential, KeyPlace, Mode, RecordError, RequestError, Secret, SecretName};
    for place in [
        KeyPlace::Query("api_key"),
        KeyPlace::Header("x-api-key"),
        KeyPlace::Bearer,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let server = Server::new();
        let spec = server.spec(
            spec(&server).quota,
            KeyUse::Required(Credential {
                tier: biomcp_types::KeyTier::Fallback,
                secret: SecretName::NcbiApiKey,
                place,
            }),
        );
        capture_credentials(&server, spec, dir.path(), place);
        let path = std::fs::read_dir(dir.path().join("pubmed"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let original = std::fs::read(&path).unwrap();
        let text = std::str::from_utf8(&original).unwrap();
        for secret in ["configured-secret", "caller-secret", "user-text"] {
            assert!(!text.contains(secret));
        }
        let value: serde_json::Value = serde_json::from_slice(&original).unwrap();
        assert_eq!(value["method"], "POST");
        assert_eq!(
            value["post"]["request_content_type"],
            "application/x-www-form-urlencoded"
        );
        assert_eq!(
            value["post"]["request_body_sha256"].as_str().unwrap().len(),
            64
        );
        let mut config = FetchConfig {
            mode: Mode::Replay {
                dir: dir.path().to_owned(),
            },
            ..FetchConfig::default()
        };
        config
            .secrets
            .insert(SecretName::NcbiApiKey, Secret::new("different-secret"));
        let replay = Fetch::open(config).unwrap();
        let get = || {
            replay
                .leg(spec, replay.deadline(Duration::from_secs(10)))
                .call(
                    &spec.post_form("/answer"),
                    biomcp_fetch::Input::Params(&[
                        ("q", Arg::Text("user-text")),
                        ("api_key", Arg::Text("caller-secret")),
                    ]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        };
        for field in ["request_content_type", "request_body_sha256"] {
            let mut changed = value.clone();
            changed["post"][field] = "wrong".into();
            std::fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
            assert_eq!(
                get().unwrap_err(),
                RequestError::Recording(RecordError::Malformed)
            );
        }
        std::fs::write(&path, original).unwrap();
        assert!(get().unwrap().answer);
        assert_eq!(
            get().unwrap_err(),
            RequestError::Recording(RecordError::Exhausted)
        );
        assert!(server.requests.try_recv().is_err());
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "Fixture construction and worker failures fail the contract"
)]
fn capture_credentials(
    server: &Server,
    spec: &'static biomcp_fetch::SourceSpec,
    dir: &std::path::Path,
    place: biomcp_fetch::KeyPlace,
) {
    use biomcp_fetch::{KeyPlace, Mode, Secret, SecretName};
    let mut config = FetchConfig {
        mode: Mode::Record {
            dir: dir.to_owned(),
        },
        ..FetchConfig::default()
    };
    config
        .secrets
        .insert(SecretName::NcbiApiKey, Secret::new("configured-secret"));
    let recording = Fetch::open(config).unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            recording
                .leg(spec, recording.deadline(Duration::from_secs(10)))
                .call(
                    &spec.post_form("/answer"),
                    biomcp_fetch::Input::Params(&[
                        ("q", Arg::Text("user-text")),
                        ("api_key", Arg::Text("caller-secret")),
                    ]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let request = server.next();
        let wire = request.text.to_ascii_lowercase();
        assert!(
            wire.contains("configured-secret"),
            "Credential routing must authenticate the wire and sanitize recordings"
        );
        match place {
            KeyPlace::Query(_) => {
                assert!(
                    request
                        .text
                        .starts_with("POST /answer?api_key=configured-secret "),
                    "Credential routing must authenticate the wire and sanitize recordings"
                );
                assert_eq!(
                    request.body, b"q=user-text",
                    "Credential routing must authenticate the wire and sanitize recordings"
                );
            }
            KeyPlace::Bearer => {
                assert!(
                    wire.contains("authorization: bearer configured-secret\r\n"),
                    "Credential routing must authenticate the wire and sanitize recordings"
                );
                assert_eq!(
                    request.body, b"q=user-text&api_key=caller-secret",
                    "Credential routing must authenticate the wire and sanitize recordings"
                );
            }
            KeyPlace::Header(_) => {
                assert!(
                    wire.contains("x-api-key: configured-secret\r\n"),
                    "Credential routing must authenticate the wire and sanitize recordings"
                );
                assert_eq!(
                    request.body, b"q=user-text&api_key=caller-secret",
                    "Credential routing must authenticate the wire and sanitize recordings"
                );
            }
        }
        request.ok();
        assert!(
            worker.join().unwrap().unwrap().answer,
            "Credential routing must authenticate the wire and sanitize recordings"
        );
    });
    drop(recording);
}
