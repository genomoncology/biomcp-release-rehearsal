//! Wake cancellable stream work without detaching its network tasks.
use crate::LocalDeadline;
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded transport state across the request owners named by this module"
)]
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

#[derive(Debug, Default)]
#[expect(
    clippy::disallowed_types,
    reason = "Acknowledge stream cancellation before releasing the worker permit"
)]
pub(super) struct Stop {
    flag: AtomicBool,
    wake: tokio::sync::Notify,
}
impl Stop {
    pub(super) fn cancel(&self) {
        self.flag.store(true, Ordering::Release);
        self.wake.notify_waiters();
    }
    pub(super) fn stopped(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }
    pub(super) async fn wait(&self) {
        loop {
            let wake = self.wake.notified();
            if self.stopped() {
                return;
            }
            wake.await;
        }
    }
}
pub(super) async fn deadline(deadline: &LocalDeadline) {
    while !deadline.expired() {
        tokio::time::sleep(
            deadline
                .end
                .saturating_duration_since(std::time::Instant::now())
                .min(std::time::Duration::from_millis(20)),
        )
        .await;
    }
}
#[derive(Debug, Default)]
#[expect(
    clippy::disallowed_types,
    reason = "Acknowledge stream cancellation before releasing the worker permit"
)]
pub(super) struct CallState {
    phase: AtomicU8,
    outcome: OnceLock<Result<(), biomcp_types::Reason>>,
    terminal: OnceLock<u8>,
}
impl CallState {
    pub(super) fn begin(&self) -> bool {
        self.phase
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
    pub(super) fn record(&self, result: Result<(), biomcp_types::Reason>) {
        self.outcome.get_or_init(|| result);
    }
    pub(super) fn outcome(&self) -> Option<Result<(), biomcp_types::Reason>> {
        self.outcome.get().copied()
    }
    pub(super) fn terminal(&self) -> Option<u8> {
        self.terminal.get().copied()
    }
    pub(super) fn set_terminal(&self, code: u8) {
        self.terminal.get_or_init(|| code);
    }
    pub(super) fn finish(&self) {
        self.phase.store(2, Ordering::Release);
    }
    fn cancel_queued(&self) -> bool {
        matches!(
            self.phase
                .compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire),
            Ok(_) | Err(2)
        )
    }
}
pub(super) struct CallGuard {
    pub(super) stop: Arc<Stop>,
    pub(super) done: std::sync::mpsc::Receiver<()>,
    pub(super) state: Arc<CallState>,
}
impl Drop for CallGuard {
    fn drop(&mut self) {
        self.stop.cancel();
        if self.state.cancel_queued() {
            return;
        }
        // The worker acknowledges only after its stream future and connection driver drop.
        // A running system resolver is the sole documented exception.
        if self.done.recv().is_err() {
            self.stop.cancel();
        }
    }
}

pub(super) async fn bounded(
    deadline: &LocalDeadline,
    call: &Stop,
    worker: &Stop,
    operation: impl std::future::Future<Output = Result<(), biomcp_types::Reason>>,
) -> Result<(), biomcp_types::Reason> {
    tokio::select! {
        biased;
        _ = call.wait() => Err(biomcp_types::Reason::DeadlineExceeded),
        _ = worker.wait() => Err(biomcp_types::Reason::Unavailable),
        _ = self::deadline(deadline) => Err(biomcp_types::Reason::DeadlineExceeded),
        result = operation => result,
    }
}
#[cfg(test)]
#[path = "dns_tests.rs"]
mod tests;
