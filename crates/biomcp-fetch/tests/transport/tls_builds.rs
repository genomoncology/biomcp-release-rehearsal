//! Count TLS settings construction without exporting transport internals.

thread_local! {
    #[expect(
        clippy::disallowed_types,
        reason = "Count configuration builds on one test thread without cross-test state"
    )]
    pub(crate) static BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[test]
fn ten_hosted_calls_build_one_tls_configuration() {
    BUILDS.with(|count| count.set(0));
    let shared = crate::Fetch::open(crate::FetchConfig::default())
        .unwrap()
        .with_hosted_secrets(crate::Secrets::default());
    for _ in 0..10 {
        let (call, cancellation) = shared.cancellable().unwrap();
        cancellation.cancel();
        drop(call);
    }
    BUILDS.with(|count| {
        assert_eq!(
            count.get(),
            1,
            "Hosted calls must share their TLS configuration"
        );
    });
}
