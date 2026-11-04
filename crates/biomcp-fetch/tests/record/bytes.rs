use super::*;
use biomcp_fetch::{Input, Read};
fn bytes(fetch: &Fetch, spec: &'static SourceSpec) -> Result<Vec<u8>, RequestError> {
    fetch
        .leg(spec, fetch.deadline(Duration::from_secs(10)))
        .call(
            &spec.get("/bytes").bytes(20),
            Input::Params(&[]),
            Read::bytes(),
        )
        .map(|received| received.value.0)
}
fn json_file(dir: &Path) -> std::path::PathBuf {
    std::fs::read_dir(dir.join("pubmed"))
        .unwrap()
        .map(|v| v.unwrap().path())
        .find(|v| v.extension().is_some_and(|v| v == "json"))
        .unwrap()
}
fn record_binary_replies(capture: &Fetch, server: &Server, spec: &'static SourceSpec) {
    thread::scope(|scope| {
        let worker = scope.spawn(|| [bytes(capture, spec), bytes(capture, spec)]);
        for payload in [b"first\x00\xff".as_slice(), b"second\x00\xfe"] {
            let mut response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: \
                close\r\n\r\n",
                payload.len()
            )
            .into_bytes();
            response.extend_from_slice(payload);
            server.next().bytes(response);
        }
        assert_eq!(
            worker.join().unwrap().map(Result::unwrap),
            [b"first\x00\xff".to_vec(), b"second\x00\xfe".to_vec()],
            "A binary capture must publish its checked body and metadata"
        );
    });
}
pub(super) fn ordered_binary_sidecars_round_trip_and_refuse_hostile_paths() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    let capture = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: dir.path().to_owned(),
        },
        max_body: 2,
        ..FetchConfig::default()
    })
    .unwrap();
    record_binary_replies(&capture, &server, spec);
    drop(capture);
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    assert_eq!(
        bytes(&replay, spec).unwrap(),
        b"first\x00\xff",
        "Binary replay must preserve order and reject unsafe sidecars"
    );
    assert_eq!(
        bytes(&replay, spec).unwrap(),
        b"second\x00\xfe",
        "Binary replay must preserve order and reject unsafe sidecars"
    );
    let path = json_file(dir.path());
    let original = std::fs::read(&path).unwrap();
    let original_value: serde_json::Value = serde_json::from_slice(&original).unwrap();
    let name = original_value
        .pointer("/exchanges/0/body/value")
        .unwrap()
        .as_str()
        .unwrap();
    refuse_hostile_sidecars(dir.path(), spec, &path, &original_value);
    // Disabled policy wins even when replay would open a hostile sidecar.
    let disabled = replay.restrict_sources([biomcp_types::Source::PubMed].into());
    assert!(
        matches!(bytes(&disabled, spec), Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::Disabled),
        "Binary replay must preserve order and reject unsafe sidecars"
    );
    assert!(
        matches!(disabled.leg(spec,
 disabled.deadline(Duration::from_secs(10)))
.call(&spec.get("/bytes").bytes(20),
Input::Params(&[]), Read::bytes().validate_body(&|_| Err(biomcp_types::Reason::BadReply))).map(|received| received.value.0),
 Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::Disabled),
        "Binary replay must preserve order and reject unsafe sidecars"
    );
    std::fs::write(&path, &original).unwrap();
    let sidecar = path.parent().unwrap().join(name);
    std::fs::remove_file(&sidecar).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&path, &sidecar).unwrap();
        let replay = fetch(Mode::Replay {
            dir: dir.path().to_owned(),
        });
        assert!(
            matches!(
                bytes(&replay, spec),
                Err(RequestError::Recording(RecordError::Malformed))
            ),
            "Binary replay must preserve order and reject unsafe sidecars"
        );
    }
    assert!(
        server.requests.try_recv().is_err(),
        "Binary replay must preserve order and reject unsafe sidecars"
    );
}

fn refuse_hostile_sidecars(
    dir: &Path,
    spec: &'static SourceSpec,
    path: &Path,
    original_value: &serde_json::Value,
) {
    for (hostile, expected) in [
        ("../outside.bin", RecordError::Malformed),
        ("/absolute.bin", RecordError::Malformed),
        ("nested/file.bin", RecordError::Malformed),
        ("missing.bin", RecordError::Missing),
        (
            "invalid\0.bin",
            RecordError::Io(std::io::ErrorKind::InvalidInput),
        ),
    ] {
        let mut value = original_value.clone();
        *value.pointer_mut("/exchanges/0/body/value").unwrap() = hostile.into();
        std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        let replay = fetch(Mode::Replay {
            dir: dir.to_owned(),
        });
        assert!(
            matches!(bytes(&replay, spec), Err(RequestError::Recording(error)) if error == expected),
            "Binary replay must preserve the failure kind for sidecar {hostile:?}"
        );
    }
}

pub(super) fn cut_binary_body_is_refused_before_returning_bytes() {
    let server = Server::new();
    let spec = spec(&server);
    let fetch = fetch(Mode::default());
    thread::scope(|scope| {
        let worker = scope.spawn(|| bytes(&fetch, spec));
        server
            .next()
            .raw("HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\nContent-Length: 19\r\nConnection: close\r\n\r\n%PDF-");
        assert!(
            matches!(worker.join().unwrap(), Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::BadReply),
            "A cut binary reply must never become a successful recording"
        );
    });
    assert!(
        server.requests.try_recv().is_err(),
        "A cut binary reply must never become a successful recording"
    );
}

pub(super) fn only_bounded_json_reports_typed_overflow() {
    for kind in ["bounded", "json", "bytes"] {
        let server = Server::new();
        let spec = spec(&server);
        let fetch = fetch(Mode::default());
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
                match kind {
                    "bounded" => leg
                        .call(
                            &spec.get("/answer"),
                            Input::Params(&[]),
                            Read::<Answer>::json().bounded(8, biomcp_types::Reason::TooLarge),
                        )
                        .map(|received| received.value)
                        .map(|_| ()),
                    "json" => leg
                        .call(
                            &spec.get("/answer").bytes(8),
                            Input::Params(&[]),
                            Read::<Answer>::json(),
                        )
                        .map(|received| received.value)
                        .map(|_| ()),
                    _ => leg
                        .call(
                            &spec.get("/answer").bytes(8),
                            Input::Params(&[]),
                            Read::bytes(),
                        )
                        .map(|received| received.value.0)
                        .map(|_| ()),
                }
            });
            let media = if kind == "bytes" {
                "application/octet-stream"
            } else {
                "application/json"
            };
            let body = "{\"answer\":true}";
            server.next().raw(&format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {media}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()
            ));
            let expected = if kind == "bounded" {
                biomcp_types::Reason::TooLarge
            } else {
                biomcp_types::Reason::BadReply
            };
            let actual = worker.join().unwrap();
            assert!(
                matches!(&actual, Err(RequestError::Source(error)) if error.reason == expected),
                "{kind}: expected {expected:?}, got {actual:?}"
            );
        });
        assert!(
            server.requests.try_recv().is_err(),
            "overflow never retries the unsized observation"
        );
    }
}
