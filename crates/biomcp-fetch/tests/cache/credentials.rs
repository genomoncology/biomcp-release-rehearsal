use super::*;
use biomcp_fetch::{Input, Read};

#[test]
fn query_identity_distinguishes_arguments_and_excludes_configured_credentials() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let credential = Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::NcbiApiKey,
        place: KeyPlace::Query("api_key"),
    };
    let spec = spec(&server, KeyUse::Required(credential));
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    let mut first = config(&path);
    first
        .secrets
        .insert(SecretName::NcbiApiKey, Secret::new("first-credential"));
    let fetch = Fetch::open(first).unwrap();
    for arg in ["a b", "a+b"] {
        exchange(&server, 200, r#"{"answer":true}"#, || {
            get(
                &fetch,
                spec,
                &endpoint,
                &[
                    ("q", Arg::Text(arg)),
                    ("api_key", Arg::Text("discard-this")),
                ],
            )
        })
        .unwrap();
    }
    let mut second = config(&path);
    second
        .secrets
        .insert(SecretName::NcbiApiKey, Secret::new("second-credential"));
    let fetch = Fetch::open(second).unwrap();
    for arg in ["a b", "a+b"] {
        assert!(
            get(&fetch, spec, &endpoint, &[("q", Arg::Text(arg))])
                .unwrap()
                .answer
        );
    }
    let connection = Connection::open(&path).unwrap();
    let urls = connection
        .prepare("SELECT url FROM reply ORDER BY url")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(urls.len(), 2);
    assert!(urls.iter().all(|url| !url.contains("credential")
        && !url.contains("api_key")
        && !url.contains("discard")));
    let missing = Fetch::open(config(&path)).unwrap();
    assert_eq!(
        source(get(&missing, spec, &endpoint, &[("q", Arg::Text("a b"))]).unwrap_err()).reason,
        Reason::NoKey
    );
    let foreign = spec.get("/answer").fresh(Freshness::Hours(1));
    let other = server.spec(spec.quota, KeyUse::None);
    assert_eq!(
        source(
            fetch
                .leg(other, fetch.deadline(Duration::from_secs(10)))
                .call(&foreign, Input::Params(&[]), Read::<Answer>::json())
                .unwrap_err()
        )
        .reason,
        Reason::Rejected
    );
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn keyed_cache_refuses_missing_credentials_and_never_masks_rejected_credentials() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(
        &server,
        KeyUse::Required(Credential {
            tier: biomcp_types::KeyTier::Fallback,
            secret: SecretName::OncokbApiKey,
            place: KeyPlace::Bearer,
        }),
    );
    let endpoint = spec.get("/answer").fresh(Freshness::Days(1));
    let mut configured = config(&path);
    configured.secrets.insert(
        SecretName::OncokbApiKey,
        Secret::new("synthetic-placeholder"),
    );
    let fetch = Fetch::open(configured).unwrap();
    assert!(fetch.has_key(SecretName::OncokbApiKey));
    assert!(
        exchange(&server, 200, r#"{"answer":true}"#, || get(
            &fetch,
            spec,
            &endpoint,
            &[]
        ))
        .unwrap()
        .answer
    );
    for mode in [Mode::default(), Mode::Offline] {
        let refused = Fetch::open(FetchConfig {
            mode,
            ..config(&path)
        })
        .unwrap();
        assert!(!refused.has_key(SecretName::OncokbApiKey));
        assert_eq!(
            source(get(&refused, spec, &endpoint, &[]).unwrap_err()).reason,
            Reason::NoKey
        );
        assert!(server.requests.try_recv().is_err());
    }
    expire(&path);
    for status in [401, 403] {
        let failure =
            exchange(&server, status, "{}", || get(&fetch, spec, &endpoint, &[])).unwrap_err();
        assert_eq!(source(failure).status, Some(status));
    }
}

#[test]
fn hosted_public_reply_cache_ignores_caller_credentials() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(
        &server,
        KeyUse::Optional(Credential {
            tier: biomcp_types::KeyTier::Fallback,
            secret: SecretName::NcbiApiKey,
            place: KeyPlace::Query("api_key"),
        }),
    );
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    let shared = Fetch::open(config(&path)).unwrap();
    for (index, key) in [Some("synthetic-public-a"), Some("synthetic-public-b"), None]
        .into_iter()
        .enumerate()
    {
        let mut secrets = biomcp_fetch::Secrets::default();
        if let Some(key) = key {
            secrets.insert(SecretName::NcbiApiKey, Secret::new(key));
        }
        secrets.insert(
            SecretName::OncokbApiKey,
            Secret::new(format!("synthetic-unrelated-{index}")),
        );
        let fetch = shared.with_hosted_secrets(secrets);
        let reply = if index == 0 {
            exchange(&server, 200, r#"{"answer":true}"#, || {
                get(&fetch, spec, &endpoint, &[])
            })
        } else {
            get(&fetch, spec, &endpoint, &[])
        };
        assert!(reply.unwrap().answer);
    }
    assert!(server.requests.try_recv().is_err());
    let count: u32 = Connection::open(path)
        .unwrap()
        .query_row("SELECT count(*) FROM reply", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}
