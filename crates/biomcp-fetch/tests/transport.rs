//! Public transport contract against isolated local servers.

use biomcp_fetch::Input;
use biomcp_fetch::{
    Arg, BadReply, Credential, Fetch, FetchConfig, KeyPlace, KeyUse, Reply, Secret, SecretName,
    SourceSpec,
};
use biomcp_types::{Reason, Source};
use serde::Deserialize;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
    time::Duration,
};

#[derive(Debug, Deserialize)]
struct Answer {
    answer: Option<String>,
}
impl Reply for Answer {
    fn check(&self) -> Result<(), BadReply> {
        self.answer
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(|_| ())
            .ok_or(BadReply)
    }
}

struct Stub {
    base: &'static str,
    requests: mpsc::Receiver<String>,
    release: Option<mpsc::Sender<()>>,
    worker: Option<thread::JoinHandle<()>>,
}
#[expect(
    clippy::unwrap_used,
    reason = "Fixture setup and worker failures must abort the contract test"
)]
impl Stub {
    fn new(status: u16, content_type: &str, body: &str, blocked: Option<&str>) -> Self {
        Self::new_at("127.0.0.1:0", status, content_type, body, blocked)
    }
    fn new_at(
        address: &str,
        status: u16,
        content_type: &str,
        body: &str,
        blocked: Option<&str>,
    ) -> Self {
        let listener = TcpListener::bind(address).unwrap();
        let base = Box::leak(format!("http://{}", listener.local_addr().unwrap()).into_boxed_str());
        let reply = format!(
            "HTTP/1.1 {status} Stub\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (sent, requests) = mpsc::channel();
        let (release, wait) = mpsc::channel();
        let blocked = blocked.map(str::to_owned);
        let worker = thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .unwrap();
                let mut request = Vec::new();
                let mut byte = [0];
                while stream.read(&mut byte).unwrap_or(0) == 1 {
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }
                if request.is_empty() {
                    break;
                }
                sent.send(String::from_utf8(request).unwrap()).unwrap();
                if blocked.as_deref() == Some("body") {
                    let (head, body) = reply.split_once("\r\n\r\n").unwrap();
                    let _ = write!(stream, "{head}\r\n\r\n");
                    let _ = wait.recv();
                    let _ = stream.write_all(body.as_bytes());
                } else {
                    if blocked.is_some() {
                        let _ = wait.recv();
                    }
                    let _ = stream.write_all(reply.as_bytes());
                }
            }
        });
        Self {
            base,
            requests,
            release: Some(release),
            worker: Some(worker),
        }
    }
    fn spec(&self, key: KeyUse, deadline: Duration) -> &'static SourceSpec {
        Box::leak(Box::new(SourceSpec {
            destinations: biomcp_fetch::Destinations::Primary,
            user_agent: None,
            terms: biomcp_fetch::Terms {
                name: "Synthetic test data",
                url: "https://example.test/terms",
                usage: biomcp_types::Use::Open,
            },
            source: Source::PubMed,
            base: self.base,
            key,
            deadline,
            quota: Box::leak(Box::new(biomcp_fetch::Quota::new(
                biomcp_fetch::SourceKey::declare("transport"),
                biomcp_fetch::Rate::per_second(1000),
                None,
                8,
            ))),
        }))
    }
    fn finish(mut self) -> Vec<String> {
        self.stop();
        self.requests.try_iter().collect()
    }
    fn stop(&mut self) {
        self.release.take();
        let _ = TcpStream::connect(self.base.trim_start_matches("http://"));
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
impl Drop for Stub {
    fn drop(&mut self) {
        self.stop();
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "Default fixture configuration must open successfully"
)]
fn fetch() -> Fetch {
    Fetch::open(FetchConfig::default()).unwrap()
}
fn credential(place: KeyPlace) -> Credential {
    Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::NcbiApiKey,
        place,
    }
}

#[test]
fn checked_json_encodes_parameters_and_sorts_identifier_sets() {
    let stub = Stub::new(
        200,
        "application/json; charset=utf-8",
        r#"{"answer":"yes"}"#,
        None,
    );
    let spec = stub.spec(KeyUse::None, Duration::from_secs(10));
    let fetch = fetch();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    let ids = ["b".into(), "a".into()];
    let reply: Answer = leg
        .call(
            &spec.get("/answer"),
            Input::Params(&[
                ("ids", Arg::Ids(&ids)),
                ("text", Arg::Text("a b&c")),
                ("n", Arg::Num(2)),
            ]),
            biomcp_fetch::Read::json(),
        )
        .map(|received| received.value)
        .unwrap();
    assert_eq!(reply.answer.as_deref(), Some("yes"));
    let requests = stub.finish();
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0].starts_with("GET /answer?ids=a%2Cb&text=a%20b%26c&n=2 HTTP/1.1"),
        "{}",
        requests[0]
    );
}

#[test]
fn http_failures_preserve_status_and_optional_404_is_absent() {
    for (status, reason) in [
        (404, Reason::NotFound),
        (400, Reason::Rejected),
        (401, Reason::Rejected),
        (429, Reason::RateLimited),
        (500, Reason::Unavailable),
        (503, Reason::Unavailable),
        (302, Reason::Rejected),
    ] {
        let stub = Stub::new(status, "application/json", "credential-echo", None);
        let spec = stub.spec(KeyUse::None, Duration::from_secs(10));
        let fetch = fetch();
        let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
        let error = leg
            .call(
                &spec.get("/answer"),
                Input::Params(&[]),
                biomcp_fetch::Read::<Answer>::json(),
            )
            .unwrap_err();
        let biomcp_fetch::RequestError::Source(error) = error else {
            panic!("expected provider failure: {error}");
        };
        assert_eq!((error.reason, error.status), (reason, Some(status)));
        if status == 404 {
            assert!(
                leg.call(
                    &spec.get("/answer"),
                    Input::Params(&[]),
                    biomcp_fetch::Read::<Answer>::json()
                )
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
        }
    }
}

#[test]
fn unusable_success_replies_are_bad_reply() {
    for (content_type, body, limit) in [
        ("text/html", "<html>error</html>", 100),
        ("application/json", "{broken", 100),
        ("application/json", r#"{"ERROR":"failed"}"#, 100),
        ("application/problem+json", r#"{"answer":"too large"}"#, 5),
    ] {
        let stub = Stub::new(200, content_type, body, None);
        let spec = stub.spec(KeyUse::None, Duration::from_secs(10));
        let fetch = Fetch::open(FetchConfig {
            max_body: limit,
            ..FetchConfig::default()
        })
        .unwrap();
        let error = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(
                &spec.get("/answer"),
                Input::Params(&[]),
                biomcp_fetch::Read::<Answer>::json(),
            )
            .unwrap_err();
        let biomcp_fetch::RequestError::Source(error) = error else {
            panic!("expected provider failure: {error}");
        };
        assert_eq!((error.reason, error.status), (Reason::BadReply, Some(200)));
    }
}

#[test]
fn octet_stream_requires_endpoint_permission_and_keeps_decode_and_body_limits() {
    for (declared, content, body, limit, accepted) in [
        (
            true,
            "binary/octet-stream",
            r#"{"answer":"yes"}"#,
            100,
            true,
        ),
        (
            false,
            "binary/octet-stream",
            r#"{"answer":"yes"}"#,
            100,
            false,
        ),
        (
            true,
            "application/octet-stream",
            r#"{"answer":"yes"}"#,
            100,
            false,
        ),
        (true, "binary/octet-stream", "{broken", 100, false),
        (true, "binary/octet-stream", r#"{"answer":"yes"}"#, 5, false),
    ] {
        let stub = Stub::new(200, content, body, None);
        let spec = stub.spec(KeyUse::None, Duration::from_secs(10));
        let fetch = Fetch::open(FetchConfig {
            max_body: limit,
            ..FetchConfig::default()
        })
        .unwrap();
        let endpoint = if declared {
            spec.get("/answer").octet_stream()
        } else {
            spec.get("/answer")
        };
        let result = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(
                &endpoint,
                Input::Params(&[]),
                biomcp_fetch::Read::<Answer>::json(),
            )
            .map(|received| received.value);
        assert_eq!(result.is_ok(), accepted);
        if let Err(biomcp_fetch::RequestError::Source(error)) = result {
            assert_eq!(error.reason, Reason::BadReply);
        }
    }
}

#[test]
fn refusals_send_zero_requests() {
    for refusal in ["no_key", "expired", "foreign"] {
        let stub = Stub::new(200, "application/json", r#"{"answer":"yes"}"#, None);
        let key = if refusal == "no_key" {
            KeyUse::Required(credential(KeyPlace::Query("api_key")))
        } else {
            KeyUse::None
        };
        let spec = stub.spec(key, Duration::from_secs(10));
        let foreign = stub.spec(KeyUse::None, Duration::from_secs(10));
        let fetch = fetch();
        let deadline = fetch.deadline(if refusal == "expired" {
            Duration::ZERO
        } else {
            Duration::from_secs(10)
        });
        let endpoint = if refusal == "foreign" {
            foreign.get("/answer")
        } else {
            spec.get("/answer")
        };
        let error = fetch
            .leg(spec, deadline)
            .call(
                &endpoint,
                Input::Params(&[]),
                biomcp_fetch::Read::<Answer>::json(),
            )
            .unwrap_err();
        let biomcp_fetch::RequestError::Source(error) = error else {
            panic!("expected provider failure: {error}");
        };
        assert_eq!(
            error.reason,
            match refusal {
                "no_key" => Reason::NoKey,
                "expired" => Reason::DeadlineExceeded,
                _ => Reason::Rejected,
            }
        );
        assert!(stub.finish().is_empty());
    }
}

#[test]
fn source_budget_caps_supplied_deadline_during_headers_and_body() {
    for (block, status) in [("head", None), ("body", Some(200))] {
        let stub = Stub::new(200, "application/json", r#"{"answer":"late"}"#, Some(block));
        let spec = stub.spec(KeyUse::None, Duration::from_secs(1));
        let fetch = fetch();
        let error = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(30)))
            .call(
                &spec.get("/answer"),
                Input::Params(&[]),
                biomcp_fetch::Read::<Answer>::json(),
            )
            .unwrap_err();
        let biomcp_fetch::RequestError::Source(error) = error else {
            panic!("expected provider failure: {error}");
        };
        assert_eq!(
            (error.reason, error.status),
            (Reason::DeadlineExceeded, status)
        );
        // The server stays blocked until after the call returns. No elapsed-time assertion.
        assert_eq!(stub.finish().len(), 1);
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "Thread-local fixture passes this test thread’s leg deadline to its Reply check"
)]
fn deadline_exhausted_during_reply_check_returns_deadline_exceeded() {
    // A per-thread timer lets the reply check finish only once its leg expires.
    thread_local! {
        static CHECK_DEADLINE: std::cell::Cell<Option<std::time::Instant>> = const {
            std::cell::Cell::new(None)
        };
    }
    #[derive(Debug, Deserialize)]
    struct CheckedAnswer {}
    impl Reply for CheckedAnswer {
        fn check(&self) -> Result<(), BadReply> {
            CHECK_DEADLINE.with(|timer| {
                let deadline = timer.get().unwrap();
                while deadline > std::time::Instant::now() {
                    thread::park_timeout(
                        deadline.saturating_duration_since(std::time::Instant::now()),
                    );
                }
            });
            Ok(())
        }
    }

    let stub = Stub::new(200, "application/json", "{}", None);
    let spec = stub.spec(KeyUse::None, Duration::from_secs(30));
    let fetch = fetch();
    let deadline = fetch.deadline(Duration::from_secs(1));
    CHECK_DEADLINE.with(|timer| timer.set(Some(std::time::Instant::now() + deadline.remaining())));
    let error = fetch
        .leg(spec, deadline)
        .call(
            &spec.get("/answer"),
            Input::Params(&[]),
            biomcp_fetch::Read::<CheckedAnswer>::json(),
        )
        .unwrap_err();
    let biomcp_fetch::RequestError::Source(error) = error else {
        panic!("expected provider failure: {error}");
    };
    CHECK_DEADLINE.with(|timer| timer.set(None));
    assert_eq!(
        (error.reason, error.status),
        (Reason::DeadlineExceeded, Some(200))
    );
    assert_eq!(stub.finish().len(), 1);
}

#[test]
fn identity_encoding_is_explicit_and_does_not_change_default_requests() {
    let stub = Stub::new(200, "application/json", r#"{"answer":"ok"}"#, None);
    let spec = stub.spec(KeyUse::None, Duration::from_secs(10));
    let fetch = fetch();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    leg.call(
        &spec.get("/answer").identity_encoding(),
        Input::Params(&[]),
        biomcp_fetch::Read::<Answer>::json(),
    )
    .map(|received| received.value)
    .unwrap();
    leg.call(
        &spec.get("/answer"),
        Input::Params(&[]),
        biomcp_fetch::Read::<Answer>::json(),
    )
    .map(|received| received.value)
    .unwrap();
    let requests = stub.finish();
    assert!(
        requests
            .first()
            .unwrap()
            .to_ascii_lowercase()
            .contains("accept-encoding: identity")
    );
    assert!(
        !requests
            .last()
            .unwrap()
            .to_ascii_lowercase()
            .contains("accept-encoding: identity")
    );
    assert_eq!(
        requests.first().unwrap().lines().next(),
        requests.last().unwrap().lines().next()
    );
}

#[test]
fn outgoing_contact_and_optional_key_never_enter_persistent_identity() {
    let stub = Stub::new(200, "application/json", r#"{"answer":"checked"}"#, None);
    let spec = stub.spec(
        KeyUse::Optional(credential(KeyPlace::Query("api_key"))),
        Duration::from_secs(10),
    );
    let recording = tempfile::tempdir().unwrap();
    let directory = recording.path().join("capture");
    let mut secrets = biomcp_fetch::Secrets::default();
    secrets.insert(SecretName::NcbiApiKey, Secret::new("placeholder-ncbi-key"));
    let fetch = Fetch::open(FetchConfig {
        secrets,
        mode: biomcp_fetch::Mode::Record {
            dir: directory.clone(),
        },
        ..FetchConfig::default()
    })
    .unwrap();
    let email = Secret::new("placeholder@example.invalid");
    for endpoint in [spec.get("/search"), spec.post_form("/summary").idempotent()] {
        let result = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(
                &endpoint,
                Input::Params(&[
                    ("tool", Arg::Text("biomcp")),
                    ("email", Arg::Contact(&email)),
                ]),
                biomcp_fetch::Read::<Answer>::json(),
            )
            .map(|received| received.value)
            .unwrap();
        assert_eq!(result.answer.as_deref(), Some("checked"));
    }
    let requests = stub.finish();
    assert_eq!(requests.len(), 2);
    for request in requests {
        assert!(request.contains("email=placeholder%40example.invalid"));
        assert!(request.contains("api_key=placeholder-ncbi-key"));
    }
    let files = std::fs::read_dir(directory.join("pubmed")).unwrap();
    for file in files {
        let bytes = std::fs::read(file.unwrap().path()).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        for forbidden in [
            "email",
            "example.invalid",
            "placeholder-ncbi-key",
            "api_key",
        ] {
            assert!(!text.contains(forbidden), "{forbidden}");
        }
    }
    assert!(!format!("{:?}", Arg::Contact(&email)).contains("placeholder"));
}

#[test]
fn bounded_reachability_probes_share_policy_and_never_cache_data() {
    let stub = Stub::new(200, "text/plain", "reachable", None);
    let spec = stub.spec(KeyUse::None, Duration::from_secs(2));
    let fetch = fetch();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(2)));
    leg.probe(&spec.get("/probe"), biomcp_fetch::Input::Params(&[]))
        .unwrap();
    leg.probe(
        &spec.post_json("/probe").idempotent(),
        biomcp_fetch::Input::json(&[], &serde_json::json!({"query":"health"})),
    )
    .unwrap();
    assert_eq!(stub.finish().len(), 2);
    let stub = Stub::new(200, "application/json", "sensitive", None);
    let spec = stub.spec(
        KeyUse::Required(credential(KeyPlace::Bearer)),
        Duration::from_secs(2),
    );
    let fetch = Fetch::open(FetchConfig {
        disabled_sources: [spec.source].into(),
        ..Default::default()
    })
    .unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(2)));
    let error = leg
        .probe(&spec.get("/probe"), biomcp_fetch::Input::Params(&[]))
        .unwrap_err();
    assert!(matches!(error, biomcp_fetch::RequestError::Source(e) if e.reason == Reason::Disabled));
    assert!(stub.finish().is_empty());
}

#[path = "transport/network.rs"]
mod network;

#[path = "transport/publisher.rs"]
mod publisher;

#[path = "transport/credentials.rs"]
mod credentials;

#[path = "transport/cancellation.rs"]
mod cancellation;
