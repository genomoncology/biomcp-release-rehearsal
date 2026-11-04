//! Source admission resolves only the required credential.
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use std::sync::mpsc;
#[test]
fn alphagenome_resolves_only_its_own_key_at_source_admission() {
    let (sent, received) = mpsc::channel();
    let mut secrets = biomcp_fetch::Secrets::default();
    secrets.defer(biomcp_fetch::SecretName::AlphaGenomeApiKey, move || {
        sent.send(()).unwrap();
        Ok(Some(biomcp_fetch::Secret::new("synthetic-alphagenome-key")))
    });
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        secrets,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(received.try_iter().count(), 0);
    let source = &biomcp_adapters::alphagenome::SPEC;
    let leg = fetch.leg(source, fetch.deadline(std::time::Duration::from_secs(1)));
    let request = biomcp_adapters::alphagenome::DnaseRequest::human_lung_all_folds(
        biodata::Build::GRCh38,
        "chr7",
        140745143..140761527,
        140753336,
        "A",
        "T",
    )
    .unwrap();
    for _ in 0..2 {
        let result = leg.grpc_checked(
            &biomcp_adapters::alphagenome::PREDICT_VARIANT,
            request.bytes(),
            |limits| request.decoder(limits),
        );
        assert!(
            matches!(result, Err(biomcp_fetch::RequestError::Source(error))
            if error.reason == biomcp_types::Reason::Offline)
        );
    }
    assert_eq!(received.try_iter().count(), 1);
}
