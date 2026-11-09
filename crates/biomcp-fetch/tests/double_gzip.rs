//! Some sources gzip a reply twice; the checked value must not depend on how many layers arrive.

use biomcp_fetch::{BadReply, Reply};
use biomcp_fetch::{Fetch, FetchConfig, Input, KeyUse, Read, RequestError, SourceSpec};
use biomcp_types::{Reason, Source};
use flate2::{Compression, write::GzEncoder};
use serde::Deserialize;
use std::{
    io::{Read as _, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};

#[derive(Debug, Deserialize)]
struct Answer {
    answer: String,
}
impl Reply for Answer {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "Fixture compression cannot fail in memory"
)]
fn gzip(body: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(body).unwrap();
    encoder.finish().unwrap()
}

/// Serve one reply with the headers a compressing source sends, and return the request seen.
#[expect(
    clippy::unwrap_used,
    reason = "Fixture setup failures must abort the test"
)]
fn serve(encoded: bool, body: Vec<u8>) -> (&'static str, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = Box::leak(format!("http://{}", listener.local_addr().unwrap()).into_boxed_str());
    let (sent, seen) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut byte = [0];
        while !request.ends_with(b"\r\n\r\n") && stream.read(&mut byte).unwrap_or(0) == 1 {
            request.push(byte[0]);
        }
        let encoding = if encoded {
            "Content-Encoding: gzip\r\n"
        } else {
            ""
        };
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{encoding}Content-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(head.as_bytes()).unwrap();
        stream.write_all(&body).unwrap();
        sent.send(String::from_utf8_lossy(&request).into_owned())
            .unwrap();
    });
    (base, seen)
}

fn spec(base: &'static str) -> &'static SourceSpec {
    Box::leak(Box::new(SourceSpec {
        destinations: biomcp_fetch::Destinations::Primary,
        user_agent: None,
        terms: biomcp_fetch::Terms {
            name: "Synthetic test data",
            url: "https://example.test/terms",
            usage: biomcp_types::Use::Open,
        },
        source: Source::UniProt,
        base,
        key: KeyUse::None,
        deadline: Duration::from_secs(10),
        quota: Box::leak(Box::new(biomcp_fetch::Quota::new(
            biomcp_fetch::SourceKey::declare("double-gzip"),
            biomcp_fetch::Rate::per_second(1000),
            None,
            8,
        ))),
    }))
}

#[expect(
    clippy::unwrap_used,
    reason = "Default fixture configuration must open"
)]
fn read(encoded: bool, body: Vec<u8>, limit: u64) -> (Result<String, RequestError>, String) {
    let (base, seen) = serve(encoded, body);
    let spec = spec(base);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    let endpoint = spec.get("/entry.json").bytes(limit).double_gzip();
    let result = leg
        .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
        .map(|received| received.value.answer);
    (
        result,
        seen.recv_timeout(Duration::from_secs(10))
            .unwrap_or_default(),
    )
}

fn reason(result: Result<String, RequestError>) -> Option<Reason> {
    match result {
        Err(RequestError::Source(error)) => Some(error.reason),
        _ => None,
    }
}

const BODY: &[u8] = br#"{"answer":"ok"}"#;

#[test]
fn twice_compressed_reply_reads_as_json_and_asks_for_gzip() {
    let (result, request) = read(true, gzip(&gzip(BODY)), 1024);
    assert_eq!(result.ok().as_deref(), Some("ok"));
    assert!(
        request
            .to_ascii_lowercase()
            .contains("accept-encoding: gzip")
    );
}

#[test]
fn reply_compressed_once_or_not_at_all_still_reads() {
    assert_eq!(read(true, gzip(BODY), 1024).0.ok().as_deref(), Some("ok"));
    assert_eq!(
        read(false, BODY.to_vec(), 1024).0.ok().as_deref(),
        Some("ok")
    );
}

#[test]
fn inner_layer_cannot_pass_the_body_limit() {
    let large = format!(r#"{{"answer":"{}"}}"#, "a".repeat(100_000));
    let (result, _) = read(true, gzip(&gzip(large.as_bytes())), 4096);
    assert_eq!(reason(result), Some(Reason::TooLarge));
}

#[test]
fn outer_layer_cannot_pass_the_body_limit() {
    let large = format!(r#"{{"answer":"{}"}}"#, "a".repeat(100_000));
    let (result, _) = read(true, gzip(large.as_bytes()), 4096);
    assert_eq!(reason(result), Some(Reason::TooLarge));
}

#[test]
fn inner_layer_reads_every_gzip_member() {
    let mut inner = gzip(br#"{"answer":"#);
    inner.extend(gzip(br#""ok"}"#));
    assert_eq!(read(true, gzip(&inner), 1024).0.ok().as_deref(), Some("ok"));
}

#[test]
fn damaged_inner_layer_is_a_bad_reply() {
    let mut inner = gzip(BODY);
    inner.truncate(inner.len() - 6);
    let (result, _) = read(true, gzip(&inner), 1024);
    assert_eq!(reason(result), Some(Reason::BadReply));
}
