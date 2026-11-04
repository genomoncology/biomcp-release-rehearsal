use super::*;
use biomcp_fetch::{Input, Read};

#[test]
fn checked_no_proxy_boundaries_select_direct_routes() {
    for (rule, direct) in [
        ("*", true),
        ("127.0.0.1", true),
        ("127.0.0.0/8", true),
        ("10.0.0.0/8", false),
        ("127.0.0.0/32", false),
        ("127.0.0.10", false),
        ("127.0.0.1.evil", false),
        (".0.0.1", false),
        (".127.0.0.1", false),
        ("localhost", true),
        (".localhost", true),
        ("*.localhost", false),
        ("port", true),
        ("wrong-port", false),
    ] {
        let stub = Stub::new(200, "application/json", r#"{"answer":"direct"}"#, None);
        let address = stub.base.trim_start_matches("http://");
        let port = address.rsplit_once(':').unwrap().1;
        let closed = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = format!(
            "http://user:synthetic-private@{}",
            closed.local_addr().unwrap()
        );
        drop(closed);
        let base = if rule.contains("localhost") {
            stub.base.replace("127.0.0.1", "localhost")
        } else {
            stub.base.into()
        };
        let rule = match rule {
            "port" => format!("127.0.0.1:{port}"),
            "wrong-port" => "127.0.0.1:0".into(),
            other => other.into(),
        };
        let network =
            biomcp_fetch::Network::new(Some(Secret::new(proxy)), None, &rule, Default::default());
        if rule == ".127.0.0.1" {
            assert!(network.is_ok());
            drop(stub);
            continue;
        }
        let fetch = Fetch::open(FetchConfig {
            network: network.unwrap(),
            ..Default::default()
        })
        .unwrap();
        let original = stub.spec(KeyUse::None, Duration::from_secs(10));
        let spec = Box::leak(Box::new(SourceSpec {
            destinations: biomcp_fetch::Destinations::Primary,
            user_agent: None,
            base: Box::leak(base.into_boxed_str()),
            source: original.source,
            terms: original.terms,
            key: original.key,
            quota: original.quota,
            deadline: original.deadline,
        }));
        let result = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(
                &spec.get("/answer"),
                Input::Params(&[]),
                Read::<Answer>::json(),
            )
            .map(|received| received.value);
        if direct {
            assert_eq!(result.unwrap().answer.as_deref(), Some("direct"));
        } else {
            let biomcp_fetch::RequestError::Source(error) = result.unwrap_err() else {
                panic!("expected source error")
            };
            assert_eq!(error.reason, Reason::Proxy, "{rule}");
        }
        assert_eq!(stub.finish().len(), usize::from(direct));
    }
}

#[test]
fn invalid_no_proxy_boundaries_are_configuration_errors() {
    for invalid in [
        "example.org:abc",
        "example.org:",
        "example.org:65536",
        "user@example.org",
        "[::1]:abc",
    ] {
        assert!(
            biomcp_fetch::Network::new(None, None, invalid, Default::default()).is_err(),
            "{invalid}"
        );
    }
    // IPv6 literals require brackets when a port is supplied.
    assert!(biomcp_fetch::Network::new(None, None, "[::1]:443", Default::default()).is_ok());
}

#[test]
fn ipv6_and_cidr_bypass_rules_route_matching_addresses_directly() {
    for (rule, direct) in [
        ("::1", true),
        ("[::1]:port", true),
        ("::/0", true),
        ("::1/128", true),
        ("::/127", true),
        ("::/128", false),
        ("::2", false),
        ("[::1]:0", false),
        ("fd00::/8", false),
        ("127.0.0.0/8", false),
    ] {
        let stub = Stub::new_at(
            "[::1]:0",
            200,
            "application/json",
            r#"{"answer":"direct"}"#,
            None,
        );
        let port = stub.base.rsplit_once(':').unwrap().1;
        let rule = rule.replace("port", port);
        let closed = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy = format!("http://{}", closed.local_addr().unwrap());
        drop(closed);
        let network =
            biomcp_fetch::Network::new(Some(Secret::new(proxy)), None, &rule, Default::default())
                .unwrap();
        let fetch = Fetch::open(FetchConfig {
            network,
            ..Default::default()
        })
        .unwrap();
        let spec = stub.spec(KeyUse::None, Duration::from_secs(10));
        let reply = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(
                &spec.get("/answer"),
                Input::Params(&[]),
                Read::<Answer>::json(),
            )
            .map(|received| received.value);
        if direct {
            assert_eq!(reply.unwrap().answer.as_deref(), Some("direct"));
        } else {
            assert!(
                matches!(reply, Err(biomcp_fetch::RequestError::Source(error))
                if error.reason == Reason::Proxy),
                "{rule}"
            );
        }
        assert_eq!(stub.finish().len(), usize::from(direct));
    }
    assert!(biomcp_fetch::Network::new(None, None, "fd00::/8", Default::default()).is_ok());
}
