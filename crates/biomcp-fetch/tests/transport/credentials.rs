//! Credential placement and reflected metadata share one transport boundary.
use super::*;
use biomcp_fetch::{Input, Read};

#[test]
fn credential_identities_scope_replies_without_disclosing_keys() {
    let name = SecretName::OncokbApiKey;
    let mut secrets = biomcp_fetch::Secrets::default();
    assert_eq!(secrets.fingerprint(name), None);
    secrets.insert(name, Secret::new(""));
    assert_eq!(secrets.fingerprint(name), None);
    secrets.insert(name, Secret::new("synthetic-caller-a"));
    let first = secrets.fingerprint(name).unwrap();
    assert_eq!(first, secrets.fingerprint(name).unwrap());
    assert_eq!(format!("{first:?}"), "KeyFingerprint(***)");
    let identity = [7; 32];
    assert_ne!(first.namespace(identity), identity);
    assert_ne!(first.namespace(identity), first.namespace([8; 32]));
    secrets.insert(name, Secret::new("synthetic-caller-b"));
    let second = secrets.fingerprint(name).unwrap();
    assert_ne!(first, second);
    assert_ne!(first.namespace(identity), second.namespace(identity));
}

#[test]
fn safe_keyed_binary_recording_replays_without_opening_http()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let stub = Stub::new(200, "application/octet-stream", "safe binary", None);
    let spec = stub.spec(
        KeyUse::Required(credential(KeyPlace::Header("x-api-key"))),
        Duration::from_secs(10),
    );
    for mode in [
        biomcp_fetch::Mode::Record {
            dir: dir.path().to_owned(),
        },
        biomcp_fetch::Mode::Replay {
            dir: dir.path().to_owned(),
        },
    ] {
        let mut config = FetchConfig {
            mode,
            ..FetchConfig::default()
        };
        config.secrets.insert(
            SecretName::NcbiApiKey,
            Secret::new("placeholder-binary-key"),
        );
        let fetch = Fetch::open(config)?;
        let bytes = fetch
            .leg(spec, fetch.deadline(Duration::from_secs(10)))
            .call(
                &spec.get("/binary").bytes(1024),
                Input::Params(&[]),
                Read::bytes(),
            )
            .map(|received| received.value.0)?;
        assert_eq!(bytes, b"safe binary");
    }
    assert_eq!(stub.finish().len(), 1);
    Ok(())
}

#[test]
fn reflected_credentials_are_sent_but_refused_without_retry_or_public_disclosure() {
    let secret = "placeholder-only-abc123";
    for place in [
        KeyPlace::Query("api_key"),
        KeyPlace::Header("x-api-key"),
        KeyPlace::Bearer,
    ] {
        let media = format!("application/json\r\nContent-Range: {secret}");
        for (media, body) in [
            ("application/json", secret),
            (media.as_str(), "safe response body"),
        ] {
            let stub = Stub::new(500, media, body, None);
            let spec = stub.spec(KeyUse::Required(credential(place)), Duration::from_secs(10));
            let mut config = FetchConfig::default();
            config
                .secrets
                .insert(SecretName::NcbiApiKey, Secret::new(secret));
            assert!(!format!("{config:?}").contains(secret));
            let fetch = Fetch::open(config).unwrap();
            let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
            let error = leg
                .call(
                    &spec.get("/answer"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .unwrap_err();
            let biomcp_fetch::RequestError::Source(error) = error else {
                panic!("expected provider failure: {error}");
            };
            for formatted in [
                format!("{fetch:?}"),
                format!("{leg:?}"),
                format!("{error:?}"),
                error.to_string(),
                Secret::new(secret).to_string(),
            ] {
                assert!(!formatted.contains(secret));
            }
            assert_eq!(error.reason, Reason::BadReply);
            let requests = stub.finish();
            assert_eq!(requests.len(), 1);
            assert!(requests[0].contains(secret));
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "Observe owned test workers without leaking shared state across cases"
)]
fn deferred_credentials_resolve_once_after_source_admission()
-> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    for disabled in [true, false] {
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let stub = Stub::new(200, "application/json", r#"{"answer":"safe"}"#, None);
        let spec = stub.spec(
            KeyUse::Optional(credential(KeyPlace::Header("x-api-key"))),
            Duration::from_secs(10),
        );
        let mut config = FetchConfig::default();
        config.secrets.defer(SecretName::NcbiApiKey, move || {
            observed.fetch_add(1, Ordering::SeqCst);
            Ok(Some(Secret::new("synthetic-deferred-key")))
        });
        if disabled {
            config.disabled_sources.insert(Source::PubMed);
        }
        assert!(!format!("{config:?}").contains("synthetic-deferred-key"));
        let fetch = Fetch::open(config)?;
        let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let reply = leg
            .call(
                &spec.get("/answer"),
                Input::Params(&[]),
                Read::<Answer>::json(),
            )
            .map(|received| received.value);
        if disabled {
            assert!(
                matches!(reply,Err(biomcp_fetch::RequestError::Source(e)) if e.reason == Reason::Disabled)
            );
            assert_eq!(calls.load(Ordering::SeqCst), 0);
        } else {
            assert_eq!(reply?.answer.as_deref(), Some("safe"));
            assert!(fetch.has_key(SecretName::NcbiApiKey));
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        }
        let requests = stub.finish();
        assert_eq!(requests.len(), usize::from(!disabled));
        if let Some(request) = requests.first() {
            assert!(request.contains("x-api-key: synthetic-deferred-key"));
        }
    }
    Ok(())
}
