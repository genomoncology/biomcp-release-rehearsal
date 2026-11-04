use super::*;

#[test]
fn publisher_destinations_refuse_private_hosts_and_applicable_proxies_before_connecting() {
    let stub = Stub::new(200, "application/pdf", "%PDF-owned", None);
    let spec = Box::leak(Box::new(SourceSpec {
        destinations: biomcp_fetch::Destinations::PublisherPdf,
        ..*stub.spec(KeyUse::None, Duration::from_secs(5))
    }));
    for url in [
        "http://127.0.0.1/file.pdf",
        "http://127.1/file.pdf",
        "http://2130706433/file.pdf",
        "http://[::1]/file.pdf",
        "https://user:secret@example.org/file.pdf",
        "file:///etc/passwd",
        "http://example.org/file.pdf#fragment",
    ] {
        assert!(spec.download(url).is_err(), "accepted {url}");
    }
    let url = stub.base.replace("127.0.0.1", "localhost");
    for (proxy, bypass, expected) in [
        (false, "", Reason::Rejected),
        (true, "", Reason::Proxy),
        (true, "localhost", Reason::Rejected),
    ] {
        let network = biomcp_fetch::Network::new(
            proxy.then(|| Secret::new(stub.base)),
            None,
            bypass,
            Default::default(),
        )
        .unwrap();
        let fetch = Fetch::open(FetchConfig {
            network,
            ..Default::default()
        })
        .unwrap();
        let deadline = fetch.deadline(Duration::from_secs(5));
        let leg = fetch.leg(spec, deadline);
        let endpoint = spec.download(&url).unwrap().bytes(1024);
        let error = leg
            .call(&endpoint, Input::Params(&[]), biomcp_fetch::Read::bytes())
            .map(|received| received.value.0)
            .unwrap_err();
        assert!(
            matches!(error, biomcp_fetch::RequestError::Source(e) if e.reason == expected),
            "{error:?}"
        );
    }
    let keyed = stub.spec(
        KeyUse::Optional(Credential {
            tier: biomcp_types::KeyTier::Fallback,
            secret: SecretName::SemanticScholarApiKey,
            place: KeyPlace::Header("x-api-key"),
        }),
        Duration::from_secs(5),
    );
    assert!(keyed.download("https://example.org/file.pdf").is_err());
    assert!(
        stub.finish().is_empty(),
        "a refused publisher destination made a request"
    );
}
