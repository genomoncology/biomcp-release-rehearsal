//! Exercise the real cancellation supervisor against a started, deliberately stalled resolver.
use super::{Stop, bounded};
#[expect(
    clippy::disallowed_types,
    reason = "Import only the counter or cell owned by this test fixture"
)]
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "Observe owned test workers without leaking shared state across cases"
)]
fn stalled_dns_returns_at_deadline_without_starting_a_stream() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .unwrap();
    let (release, wait) = mpsc::sync_channel(0);
    let (started, running) = tokio::sync::oneshot::channel();
    let (finished, done) = mpsc::sync_channel(1);
    let stream_started = Arc::new(AtomicBool::new(false));
    let observed = stream_started.clone();
    let result = runtime.block_on(async {
        let operation = async move {
            super::super::dns::resolve(move || {
                started.send(()).unwrap();
                wait.recv().unwrap();
                finished.send(()).unwrap();
                Ok(([127, 0, 0, 1], 443).into())
            })
            .await?;
            observed.store(true, Ordering::Release);
            Ok(())
        };
        tokio::pin!(operation);
        // Start the real blocking slot before starting the deadline. No sleeping guess.
        tokio::select! {
            _ = &mut operation => panic!("resolver unexpectedly completed"),
            _ = running => {},
        }
        let deadline = crate::Deadline::start(Duration::from_millis(50), None).local();
        bounded(
            &deadline,
            &Stop::default(),
            &Stop::default(),
            &mut operation,
        )
        .await
    });
    assert_eq!(result, Err(biomcp_types::Reason::DeadlineExceeded));
    assert!(!stream_started.load(Ordering::Acquire));
    runtime.shutdown_timeout(Duration::ZERO);
    // The one approved detached system lookup finishes after its controlled release.
    release.send(()).unwrap();
    done.recv_timeout(Duration::from_secs(5)).unwrap();
}
