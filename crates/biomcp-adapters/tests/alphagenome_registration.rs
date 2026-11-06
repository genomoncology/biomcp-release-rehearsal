//! Feature alphagenome-registration: source admission retains required-key policy and terms.
use biomcp_adapters::{alphagenome::SPEC, registry};
use biomcp_fetch::{Credential, Fetch, FetchConfig, KeyPlace, KeyUse, SecretName};
use biomcp_types::{Reason, Source, Use};
#[test]
fn restricted_source_requires_its_own_key_before_any_provider_request()
-> Result<(), Box<dyn std::error::Error>> {
    let source = Source::AlphaGenome;
    let terms = registry::terms(source).ok_or("missing terms")?;
    assert_eq!(terms.usage, Use::Licensed);
    assert_eq!(terms.url, "https://alphagenome.google/terms");
    assert_eq!(SPEC.base, "https://gdmscience.googleapis.com");
    assert!(matches!(
        SPEC.key,
        KeyUse::Required(Credential {
            tier: biomcp_types::KeyTier::NoFallback,
            secret: SecretName::AlphaGenomeApiKey,
            place: KeyPlace::Header("x-goog-api-key")
        })
    ));
    assert!(registry::disabled(&["licensed".into()])?.contains(&source));
    let fetch = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })?;
    let request = biomcp_adapters::alphagenome::DnaseRequest::human_lung_all_folds(
        biodata::Build::GRCh38,
        "chr7",
        140745143..140761527,
        140753336,
        "A",
        "T",
    )?;
    let leg = fetch.leg(&SPEC, fetch.deadline(std::time::Duration::from_secs(1)));
    let result = leg.grpc_checked(
        &biomcp_adapters::alphagenome::PREDICT_VARIANT,
        request.bytes(),
        |limits| request.decoder(limits),
    );
    assert!(
        matches!(result,Err(biomcp_fetch::RequestError::Source(error)) if error.reason == Reason::NoKey && error.source == source)
    );
    Ok(())
}
