//! Feature alphagenome-protocol: route placeholder credentials only to the local TLS peer.
use super::{Check, QUOTA, support};
const METHOD: &str = "/synthetic.Stream/Exchange";
const SCHEMA: &str = "synthetic-stream-v1";
use biomcp_fetch::{
    Credential, Fetch, FetchConfig, Freshness, KeyPlace, KeyUse, Secret, SecretName, Secrets,
    SourceSpec,
};
use biomcp_types::{Reason, Source};
use std::{sync::atomic::Ordering, time::Duration};
#[test]
fn required_key_refuses_without_connections_and_keyed_request_uses_metadata() -> Check {
    let fixture =
        support::Fixture::with_messages(support::Hold::None, 0, false, false, vec![vec![1, 2, 3]])?;
    let source = Box::leak(Box::new(SourceSpec {
        source: Source::AlphaGenome,
        destinations: biomcp_fetch::Destinations::Primary,
        user_agent: None,
        terms: biomcp_fetch::Terms {
            name: "Synthetic terms",
            url: "https://example.test/terms",
            usage: biomcp_types::Use::Licensed,
        },
        base: Box::leak(format!("https://127.0.0.1:{}", fixture.address.port()).into_boxed_str()),
        key: KeyUse::Required(Credential {
            tier: biomcp_types::KeyTier::Fallback,
            secret: SecretName::AlphaGenomeApiKey,
            place: KeyPlace::Header("x-goog-api-key"),
        }),
        quota: &QUOTA,
        deadline: Duration::from_secs(5),
    }));
    let endpoint = source.grpc(METHOD).checked(SCHEMA, Freshness::Never);
    for keyed in [false, true] {
        let mut secrets = Secrets::default();
        if keyed {
            secrets.insert(
                SecretName::AlphaGenomeApiKey,
                Secret::new("synthetic-alphagenome-placeholder"),
            );
        }
        let fetch = Fetch::open(FetchConfig {
            secrets,
            network: fixture.network(None, "")?,
            ..Default::default()
        })?;
        let leg = fetch.leg(source, fetch.deadline(Duration::from_secs(5)));
        let result = leg.grpc(&endpoint, &[4, 5], |_| Ok(()));
        if keyed {
            result?;
        } else {
            assert!(
                matches!(result,Err(biomcp_fetch::RequestError::Source(error)) if error.reason == Reason::NoKey && error.source == Source::AlphaGenome)
            );
            assert_eq!(fixture.count.load(Ordering::SeqCst), 0);
            assert_eq!(
                SecretName::AlphaGenomeApiKey.environment(),
                "ALPHAGENOME_API_KEY"
            );
        }
    }
    loop {
        if let support::Seen::Request(headers, body) = fixture.received()? {
            assert_eq!(
                headers.get("x-goog-api-key").and_then(|v| v.to_str().ok()),
                Some("synthetic-alphagenome-placeholder")
            );
            assert!(!headers.contains_key("authorization"));
            assert_eq!(body.get(5..), Some(&[4, 5][..]));
            break;
        }
    }
    assert_eq!(fixture.count.load(Ordering::SeqCst), 1);
    fixture.finish()
}
