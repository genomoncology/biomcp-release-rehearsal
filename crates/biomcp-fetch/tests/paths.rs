//! Feature outbound-hosts: source paths preserve ownership before storage or network admission.
#![expect(
    clippy::disallowed_methods,
    reason = "Local public transport fixture owns capture and cache files"
)]
mod support;
use biomcp_fetch::{
    Fetch, FetchConfig, Freshness, Input, InputError, KeyUse, Mode, Quota, Rate, Read, SourceKey,
};
use std::{thread, time::Duration};
use support::Answer;
type Server = support::Server<()>;
#[test]
fn runtime_paths_share_literal_capture_and_cache_identity_without_host_override() {
    let server = Server::new();
    let spec = server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("paths"),
            Rate::per_second(1000),
            None,
            4,
        ))),
        KeyUse::None,
    );
    let literal = spec
        .get("/mapping/P15056/pathways")
        .fresh(Freshness::Days(7));
    let dynamic = spec
        .get("/mapping")
        .fresh(Freshness::Days(7))
        .with_path(format!("/mapping/{}/pathways", "P15056"))
        .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let capture = directory.path().join("capture");
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: capture.clone(),
        },
        ..Default::default()
    })
    .unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(10)))
                .call(&dynamic, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value)
        });
        let request = server.next();
        assert!(
            request
                .text
                .starts_with("GET /mapping/P15056/pathways HTTP/1.1\r\n")
        );
        request.ok();
        assert!(worker.join().unwrap().unwrap().answer);
    });
    drop(fetch);
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: capture },
        ..Default::default()
    })
    .unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    assert!(
        leg.call(&literal, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    assert!(
        leg.call(&dynamic, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    let different = spec
        .get("/mapping")
        .with_path("/mapping/P04637/pathways")
        .unwrap();
    assert!(matches!(
        leg.call(&different, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value),
        Err(biomcp_fetch::RequestError::Recording(
            biomcp_fetch::RecordError::Missing
        ))
    ));
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn runtime_paths_reject_other_url_parts_and_noncanonical_segments() {
    let server = Server::new();
    let quota = Box::leak(Box::new(Quota::new(
        SourceKey::declare("path-validation"),
        Rate::per_second(1),
        None,
        1,
    )));
    let spec = server.spec(quota, KeyUse::None);
    for path in [
        "https://other.example/x",
        "//other.example/x",
        "/x?host=other",
        "/x#other",
        "/../x",
        "/x/./y",
        "/x%2Fy",
        "/x\\y",
        "/x\ny",
    ] {
        assert!(
            matches!(spec.get("/mapping").with_path(path), Err(InputError::Path)),
            "{path:?}"
        );
    }
}

#[test]
fn raw_segments_encode_identifier_delimiters_once_and_preserve_literal_identity() {
    use biomcp_fetch::PathSegment;
    let server = Server::new();
    let quota = Box::leak(Box::new(Quota::new(
        SourceKey::declare("encoded-path"),
        Rate::per_second(1000),
        None,
        1,
    )));
    let spec = server.spec(quota, KeyUse::None);
    let endpoint = spec.get("/variant/get").with_segments(&[
        PathSegment::new("litvar@rs121913530##/piece").unwrap(),
        PathSegment::new("publications").unwrap(),
    ]);
    let directory = tempfile::tempdir().unwrap();
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: directory.path().join("record"),
        },
        ..Default::default()
    })
    .unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(10)))
                .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value)
        });
        let request = server.next();
        assert!(request.text.starts_with(
            "GET /variant/get/litvar%40rs121913530%23%23%2Fpiece/publications HTTP/1.1\r\n"
        ));
        request.ok();
        assert!(worker.join().unwrap().unwrap().answer);
    });
    drop(fetch);
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: directory.path().join("record"),
        },
        ..Default::default()
    })
    .unwrap();
    let literal = spec.get("/variant/get/litvar%40rs121913530%23%23%2Fpiece/publications");
    assert!(
        fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(&literal, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    for invalid in [
        "",
        ".",
        "..",
        "https://other/x",
        "//other",
        "x\\y",
        "x%40y",
        "x\ny",
        "../x",
        "x/../y",
        "x/./y",
    ] {
        assert!(
            matches!(PathSegment::new(invalid), Err(InputError::Path)),
            "{invalid:?}"
        );
    }
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn undeclared_download_host_is_refused_before_replay_admission() {
    let server = Server::new();
    let spec = server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("host-admission"),
            Rate::per_second(1),
            None,
            1,
        ))),
        KeyUse::None,
    );
    let directory = tempfile::tempdir().unwrap();
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: directory.path().to_owned(),
        },
        ..Default::default()
    })
    .unwrap();
    let endpoint = spec.download("https://undeclared.example/data").unwrap();
    let error = fetch
        .leg(spec, fetch.deadline(Duration::from_secs(1)))
        .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
        .unwrap_err();
    assert_eq!(error.to_string(), "fetch.input_host");
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn undeclared_origin_refuses_live_and_offline_without_contacting_the_local_peer() {
    let server = Server::new();
    let spec = server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("host-boundary"),
            Rate::per_second(1),
            None,
            1,
        ))),
        KeyUse::None,
    );
    for mode in [Mode::default(), Mode::Offline] {
        let fetch = Fetch::open(FetchConfig {
            mode,
            ..Default::default()
        })
        .unwrap();
        // A literal path must not reinterpret the base authority as URL credentials.
        let endpoint = spec.get(Box::leak(
            format!(
                "@localhost:{}/answer",
                server.base.rsplit(':').next().unwrap()
            )
            .into_boxed_str(),
        ));
        assert!(matches!(
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(1)))
                .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value),
            Err(biomcp_fetch::RequestError::Input(InputError::Host))
        ));
    }
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn declared_secondary_origins_are_exact_and_publisher_policy_is_explicit() {
    let server = Server::new();
    let template = server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("secondary-hosts"),
            Rate::per_second(1000),
            None,
            1,
        ))),
        KeyUse::None,
    );
    let spec = Box::leak(Box::new(biomcp_fetch::SourceSpec {
        destinations: biomcp_fetch::Destinations::Additional(&["https://cdn.example"]),
        ..*template
    }));
    let directory = tempfile::tempdir().unwrap();
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: directory.path().to_owned(),
        },
        ..Default::default()
    })
    .unwrap();
    for (url, admitted) in [
        ("https://cdn.example/file", true),
        ("https://cdn.example:443/file", true),
        ("https://cdn.example.other/file", false),
        ("https://sub.cdn.example/file", false),
        ("http://cdn.example/file", false),
        ("https://cdn.example:444/file", false),
    ] {
        let endpoint = spec.download(url).unwrap();
        let error = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(1)))
            .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            if admitted {
                "fetch.record_missing"
            } else {
                "fetch.input_host"
            },
            "{url}"
        );
    }
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn grpc_host_admission_precedes_worker_submission_and_checked_replay() {
    use biomcp_fetch::grpc::{Limits, StreamDecoder};
    struct Decoder;
    impl StreamDecoder for Decoder {
        type Reply = ();
        fn message(&mut self, _: &[u8]) -> Result<(), biomcp_fetch::BadReply> {
            Ok(())
        }
        fn finish(self) -> Result<(), biomcp_fetch::BadReply> {
            Ok(())
        }
    }
    let server = Server::new();
    let template = server.spec(
        Box::leak(Box::new(Quota::new(
            SourceKey::declare("grpc-hosts"),
            Rate::per_second(1),
            None,
            1,
        ))),
        KeyUse::None,
    );
    let spec = Box::leak(Box::new(biomcp_fetch::SourceSpec {
        base: "https://*.localhost",
        ..*template
    }));
    let endpoint = spec
        .grpc("/synthetic.Service/Read")
        .checked("synthetic/v1", Freshness::Never);
    let directory = tempfile::tempdir().unwrap();
    for mode in [
        Mode::default(),
        Mode::Replay {
            dir: directory.path().to_owned(),
        },
    ] {
        let live = matches!(mode, Mode::Live { .. });
        let fetch = Fetch::open(FetchConfig {
            mode,
            ..Default::default()
        })
        .unwrap();
        let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(1)));
        if live {
            assert!(matches!(
                leg.grpc(&endpoint, &[], |_| Ok(())),
                Err(biomcp_fetch::RequestError::Input(InputError::Host))
            ));
        }
        assert!(matches!(
            leg.grpc_checked(&endpoint, &[], |_: Limits| Ok(Decoder)),
            Err(biomcp_fetch::RequestError::Input(InputError::Host))
        ));
    }
    assert!(server.requests.try_recv().is_err());
}
