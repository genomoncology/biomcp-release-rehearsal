//! Prove joined shutdown without exporting a test-only transport control.
use crate::{
    Certificates, Fetch, FetchConfig, KeyUse, Network, Quota, Rate, SourceKey, SourceSpec, Terms,
};
use biomcp_test_support::grpc as support;
use std::time::Duration;
use support::{Check, Fixture, Hold};
static QUOTA: Quota = Quota::new(
    SourceKey::declare("grpc-shutdown"),
    Rate::per_second(100),
    None,
    1,
);
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "The private shutdown test scopes its caller so every exit joins network work"
)]
fn stream_worker_shutdown_cancels_and_joins_active_network_work() -> Check {
    let fixture = Fixture::new(Hold::Headers, 0, false, false)?;
    let fetch = Fetch::open(FetchConfig {
        network: Network::new(
            None,
            None,
            "",
            Certificates::from_pem(fixture.certificate.as_bytes())?,
        )?,
        ..Default::default()
    })?;
    let source: &'static SourceSpec = Box::leak(Box::new(SourceSpec {
        source: biomcp_types::Source::MyGene,
        destinations: crate::Destinations::Primary,
        user_agent: None,
        base: Box::leak(format!("https://{}", fixture.address).into_boxed_str()),
        terms: Terms {
            name: "Synthetic fixture",
            url: "https://example.test/terms",
            usage: biomcp_types::Use::Open,
        },
        key: KeyUse::None,
        quota: &QUOTA,
        deadline: Duration::from_secs(5),
    }));
    std::thread::scope(|scope| {
        let caller = fetch.clone();
        let call = scope.spawn(move || {
            caller
                .leg(source, caller.deadline(Duration::from_secs(5)))
                .grpc(&source.grpc("/synthetic.Tensor/Exchange"), &[], |_| Ok(()))
        });
        while !matches!(fixture.received()?, support::Seen::Request(_, _)) {}
        fetch.grpc_worker.shutdown();
        assert!(
            call.join().map_err(|_| "caller panicked")?.is_err(),
            "shutdown did not cancel the caller"
        );
        assert!(
            fetch
                .leg(source, fetch.deadline(Duration::from_secs(1)))
                .grpc(&source.grpc("/synthetic.Tensor/Exchange"), &[], |_| Ok(()))
                .is_err(),
            "closed worker accepted another request"
        );
        assert_eq!(
            fixture.count.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "closed worker opened another socket"
        );
        fixture.finish()
    })
}
