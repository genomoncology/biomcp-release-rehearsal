//! Checked XML uses the shared request, cache and recording boundaries.
#![expect(
    clippy::disallowed_methods,
    clippy::unwrap_used,
    reason = "Local HTTP fixtures and scoped public API callers"
)]
use biomcp_fetch::{Input, Read};

mod support;
use biomcp_fetch::{
    Arg, BadReply, Fetch, FetchConfig, Freshness, KeyUse, Quota, Rate, Reply, SourceKey,
    xml::{MixedText, XmlReply},
};
use std::{thread, time::Duration};
use support::Answer as JsonAnswer;
type Server = support::Server;
#[derive(Debug, serde::Deserialize)]
struct Answer {
    text: MixedText,
}
impl XmlReply for Answer {
    fn decode_xml(xml: &str) -> Result<Self, BadReply> {
        Ok(Self {
            text: MixedText::from_xml(xml)?,
        })
    }
}
impl Reply for Answer {
    fn check(&self) -> Result<(), BadReply> {
        if self.text.as_str().contains("V600E") {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}
fn spec(server: &Server) -> &'static biomcp_fetch::SourceSpec {
    server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("xml"),
            Rate::per_second(1000),
            None,
            4,
        ))),
        KeyUse::None,
    )
}
fn respond(request: support::Request<()>, media: &str, body: &[u8]) {
    let mut response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {media}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    request.bytes(response);
}
#[test]
fn checked_get_and_form_xml_preserve_text_and_select_media() {
    let server = Server::new();
    let spec = spec(&server);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    for (ep, media, body) in [
        (
            spec.get("/answer"),
            "Application/XML; Charset=\"UTF-8\"",
            "\u{feff}<?xml version=\"1.0\" encoding=\"UTF-8\"?><answer>BRAF<sup>V600E</sup></answer>",
        ),
        (
            spec.post_form("/answer"),
            "application/provider+xml; version=1",
            "<!DOCTYPE answer SYSTEM \"http://127.0.0.1:1/never\"><answer>BRAF<sup>V600E</sup></answer>",
        ),
        (
            spec.get("/answer"),
            "text/xml",
            "<answer>BRAF<sup>V600E</sup></answer>\n<!--tail-->",
        ),
    ] {
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(10)))
                    .call(
                        &ep,
                        Input::Params(&[("q", Arg::Num(1))]),
                        Read::<Answer>::xml(),
                    )
                    .map(|received| received.value)
            });
            let request = server.next();
            assert!(
                request
                    .text
                    .to_ascii_lowercase()
                    .contains("accept: application/xml\r\n")
            );
            if request.text.starts_with("POST") {
                assert_eq!(request.body, b"q=1");
            }
            respond(request, media, body.as_bytes());
            assert_eq!(worker.join().unwrap().unwrap().text.as_str(), "BRAFV600E");
        });
    }
}

fn exchange(
    server: &Server,
    fetch: &Fetch,
    spec: &'static biomcp_fetch::SourceSpec,
    media: &str,
    body: &[u8],
) -> Result<Answer, biomcp_fetch::RequestError> {
    thread::scope(|scope| {
        let worker = scope.spawn(|| invoke(fetch, spec));
        respond(server.next(), media, body);
        worker.join().unwrap()
    })
}
fn invoke(
    fetch: &Fetch,
    spec: &'static biomcp_fetch::SourceSpec,
) -> Result<Answer, biomcp_fetch::RequestError> {
    invoke_leg(
        &fetch.leg(spec, fetch.deadline(Duration::from_secs(10))),
        spec,
    )
}
fn invoke_leg(
    leg: &biomcp_fetch::Leg<'_>,
    spec: &'static biomcp_fetch::SourceSpec,
) -> Result<Answer, biomcp_fetch::RequestError> {
    leg.call(
        &spec.get("/answer").fresh(Freshness::Days(1)),
        Input::Params(&[]),
        Read::xml(),
    )
    .map(|received| received.value)
}
#[test]
fn invalid_xml_media_encodings_documents_and_reply_checks_are_input_free() {
    use biomcp_types::Reason;
    let server = Server::new();
    let spec = spec(&server);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    for (media, body) in [
        ("application/json", "<answer>V600E</answer>"),
        ("text/html;charset=ISO-8859-1", "<answer>V600E</answer>"),
        ("text/html+xml", "<answer>V600E</answer>"),
        ("text/xml; charset=iso-8859-1", "<answer>V600E</answer>"),
        (
            "text/xml; charset=utf-8; charset=utf-16",
            "<answer>V600E</answer>",
        ),
        (
            "text/xml",
            "<?xml version=\"1.0\" encoding=\"UTF-16\"?><answer>V600E</answer>",
        ),
        (
            "text/xml",
            "<?xml version=\"1.0\" encoding=\"UTF-8\" encoding=\"UTF-16\"?><answer>V600E</answer>",
        ),
        (
            "text/xml",
            "<?xml version=\"1.0\" standalone=\"yes\" encoding=\"UTF-8\"?><answer>V600E</answer>",
        ),
        (
            "text/xml",
            "<?xml version=\"1.0\" unknown=\"private-error\"?><answer>V600E</answer>",
        ),
        ("text/xml", "<answer>V600E"),
        ("text/xml", "<answer>V600E</wrong>"),
        ("text/xml", "<answer>V600E</answer><extra/>"),
        ("text/xml", "<answer>V600E</answer>private-error"),
        (
            "text/xml",
            "<answer><![CDATA[V600E]]></answer><![CDATA[tail]]>",
        ),
        ("text/xml", "<answer>V600E&custom;</answer>"),
        (
            "text/xml",
            "<!DOCTYPE answer [<!ENTITY custom 'V600E'>]><answer>&custom;</answer>",
        ),
        ("text/xml", "<answer attr=\"&custom;\">V600E</answer>"),
        ("text/xml", "<answer>V600E<!--bad--comment--></answer>"),
        ("text/xml", "<answer>private-error</answer>"),
        ("text/xml", "<answer>V600E</answer><!DOCTYPE answer>"),
        (
            "text/xml",
            "<!DOCTYPE answer><!DOCTYPE answer><answer>V600E</answer>",
        ),
        ("text/xml", " <?xml version=\"1.0\"?><answer>V600E</answer>"),
        (
            "text/xml",
            "<?xml version=\"1.0\"?><?xml version=\"1.0\"?><answer>V600E</answer>",
        ),
        ("text/xml", "&amp;<answer>V600E</answer>"),
        ("text/xml", ""),
    ] {
        let error = exchange(&server, &fetch, spec, media, body.as_bytes()).unwrap_err();
        assert!(
            matches!(error, biomcp_fetch::RequestError::Source(e) if e.reason == Reason::BadReply),
            "{body}: {error}"
        );
        assert!(!format!("{error:?} {error}").contains("private-error"));
    }
    let error = exchange(
        &server,
        &fetch,
        spec,
        "text/xml",
        b"<answer>V600E\xff</answer>",
    )
    .unwrap_err();
    assert!(matches!(error, biomcp_fetch::RequestError::Source(e) if e.reason == Reason::BadReply));
}
#[test]
fn xml_body_and_nested_inline_content_are_bounded() {
    use biomcp_types::Reason;
    let server = Server::new();
    let spec = spec(&server);
    for (max_body, body) in [
        (16, "<answer>V600E</answer>".into()),
        (
            16 * 1024,
            format!(
                "<answer>{}V600E{}</answer>",
                "<i>".repeat(80),
                "</i>".repeat(80)
            ),
        ),
    ] {
        let fetch = Fetch::open(FetchConfig {
            max_body,
            ..FetchConfig::default()
        })
        .unwrap();
        let error = exchange(&server, &fetch, spec, "text/xml", body.as_bytes()).unwrap_err();
        assert!(
            matches!(error, biomcp_fetch::RequestError::Source(e) if e.reason == Reason::BadReply)
        );
    }
    // Arbitrary nested inline content below the bound is accepted.
    let body = format!(
        "<answer>{}V600E{}</answer>",
        "<i>".repeat(40),
        "</i>".repeat(40)
    );
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    assert!(exchange(&server, &fetch, spec, "text/xml", body.as_bytes()).is_ok());
}

#[test]
fn xml_cache_codec_invalid_entries_and_stale_fallback_remain_checked() {
    use biomcp_fetch::{Mode, RequestError};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server);
    let fetch = Fetch::open(FetchConfig {
        cache: Some(path.clone()),
        ..FetchConfig::default()
    })
    .unwrap();
    let body = b"<answer>BRAF<sup>V600E</sup></answer>";
    assert!(exchange(&server, &fetch, spec, "text/xml", body).is_ok());
    assert_eq!(invoke(&fetch, spec).unwrap().text.as_str(), "BRAFV600E");
    assert!(server.requests.try_recv().is_err());
    let database = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        database
            .query_row("SELECT content_type FROM reply", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "application/xml"
    );
    for sql in [
        "UPDATE reply SET content_type='application/unknown'",
        "UPDATE reply SET content_type='application/json'",
        "UPDATE reply SET body=CAST('<broken>' AS BLOB)",
        "UPDATE reply SET body=CAST('<answer>unchecked</answer>' AS BLOB)",
    ] {
        database.execute_batch(sql).unwrap();
        assert!(exchange(&server, &fetch, spec, "text/xml", body).is_ok());
    }
    database
        .execute_batch("UPDATE reply SET fresh_until=0")
        .unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    thread::scope(|scope| {
        let worker = scope.spawn(|| invoke_leg(&leg, spec));
        server.next().respond(403, "", "unchecked-secret");
        assert_eq!(worker.join().unwrap().unwrap().text.as_str(), "BRAFV600E");
    });
    assert!(leg.finish().stale);
    database
        .execute_batch("UPDATE reply SET body=CAST('<broken>' AS BLOB)")
        .unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| invoke(&fetch, spec));
        server.next().respond(403, "", "unchecked-secret");
        assert!(
            matches!(worker.join().unwrap(), Err(RequestError::Source(e)) if e.status == Some(403))
        );
    });
    assert_eq!(
        database
            .query_row("SELECT COUNT(*) FROM reply", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        0
    );
    assert!(exchange(&server, &fetch, spec, "text/xml", body).is_ok());
    // A caller selecting JSON for the same request replaces the XML entry.
    let endpoint = spec.get("/answer").fresh(Freshness::Days(1));
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(10)))
                .call(&endpoint, Input::Params(&[]), Read::<JsonAnswer>::json())
                .map(|received| received.value)
        });
        server.next().ok();
        assert!(worker.join().unwrap().unwrap().answer);
    });
    let offline = Fetch::open(FetchConfig {
        cache: Some(path),
        mode: Mode::Offline,
        ..FetchConfig::default()
    })
    .unwrap();
    assert!(
        matches!(invoke(&offline, spec), Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::Offline)
    );
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn record_xml_text_replays_without_http_and_offline_uses_checked_codec() {
    use biomcp_fetch::{Mode, ReplayCache};
    let dir = tempfile::tempdir().unwrap();
    let capture = dir.path().join("capture");
    let path = dir.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server);
    let config = || FetchConfig {
        cache: Some(path.clone()),
        ..FetchConfig::default()
    };
    let live = Fetch::open(config()).unwrap();
    let body = b"<answer>BRAF<sup>V600E</sup></answer>";
    exchange(&server, &live, spec, "text/xml", body).unwrap();
    let record = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: capture.clone(),
        },
        ..config()
    })
    .unwrap();
    // Record ignores the already warm cache and retains the original XML text.
    exchange(&server, &record, spec, "text/xml", body).unwrap();
    drop(record);
    let file = std::fs::read_dir(capture.join("pubmed"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let recording: serde_json::Value =
        serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    assert!(recording.to_string().contains("<sup>V600E</sup>"));
    drop(server);
    let case_cache = ReplayCache::temporary().unwrap();
    let replay = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: capture },
        replay_cache: Some(case_cache.clone()),
        ..config()
    })
    .unwrap();
    assert_eq!(invoke(&replay, spec).unwrap().text.as_str(), "BRAFV600E");
    assert_eq!(invoke(&replay, spec).unwrap().text.as_str(), "BRAFV600E");
    let offline = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        replay_cache: Some(case_cache),
        ..FetchConfig::default()
    })
    .unwrap();
    assert_eq!(invoke(&offline, spec).unwrap().text.as_str(), "BRAFV600E");
    // Offline accepts a valid stale XML entry and reports its freshness.
    let database = rusqlite::Connection::open(&path).unwrap();
    database
        .execute_batch("UPDATE reply SET fresh_until=0")
        .unwrap();
    let offline = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        ..config()
    })
    .unwrap();
    let leg = offline.leg(spec, offline.deadline(Duration::from_secs(10)));
    assert!(
        leg.call(
            &spec.get("/answer").fresh(Freshness::Days(1)),
            Input::Params(&[]),
            Read::<Answer>::xml()
        )
        .map(|received| received.value)
        .is_ok()
    );
    assert!(leg.finish().stale);
}

impl XmlReply for support::Answer {}
#[test]
fn ordinary_xml_replies_use_default_serde_decoding() {
    let server = Server::new();
    let spec = spec(&server);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(10)))
                .call(
                    &spec.get("/plain"),
                    Input::Params(&[]),
                    Read::<JsonAnswer>::xml(),
                )
                .map(|received| received.value)
        });
        respond(
            server.next(),
            "application/xml",
            b"<reply><answer>true</answer></reply>\n<!--tail-->",
        );
        assert!(worker.join().unwrap().unwrap().answer);
    });
}

#[test]
fn application_octet_stream_permission_keeps_xml_checks_and_bounds() {
    for (declared, media, body, limit, accepted) in [
        (
            false,
            "application/octet-stream",
            "<answer>V600E</answer>",
            100,
            false,
        ),
        (
            true,
            "application/octet-stream",
            "<answer>V600E</answer>",
            100,
            true,
        ),
        (
            true,
            "binary/octet-stream",
            "<answer>V600E</answer>",
            100,
            false,
        ),
        (true, "application/xml", "<answer>V600E</answer>", 100, true),
        (true, "application/octet-stream", "<answer>", 100, false),
        (
            true,
            "application/octet-stream",
            "<answer>unrelated</answer>",
            100,
            false,
        ),
        (
            true,
            "application/octet-stream",
            "<answer>V600E</answer>",
            5,
            false,
        ),
    ] {
        let server = Server::new();
        let spec = spec(&server);
        let fetch = Fetch::open(FetchConfig {
            max_body: limit,
            ..FetchConfig::default()
        })
        .unwrap();
        let endpoint = if declared {
            spec.get("/gpml").application_octet_stream()
        } else {
            spec.get("/gpml")
        };
        thread::scope(|scope| {
            let worker = scope.spawn(|| {
                fetch
                    .leg(spec, fetch.deadline(Duration::from_secs(10)))
                    .call(&endpoint, Input::Params(&[]), Read::<Answer>::xml())
                    .map(|received| received.value)
            });
            respond(server.next(), media, body.as_bytes());
            assert_eq!(worker.join().unwrap().is_ok(), accepted);
        });
    }
}
