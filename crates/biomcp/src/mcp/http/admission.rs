//! Bound framing readers separately from active requests so health stays reachable.
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded HTTP admission counts across connection workers"
)]
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[expect(
    clippy::disallowed_types,
    reason = "Bound HTTP framing readers separately from active requests"
)]
#[derive(Default)]
pub(super) struct Slots<const LIMIT: usize>(Arc<AtomicUsize>);
#[expect(
    clippy::disallowed_types,
    reason = "Release one HTTP admission count when its worker leaves the phase"
)]
pub(super) struct Permit(Arc<AtomicUsize>);
impl<const LIMIT: usize> Slots<LIMIT> {
    pub(super) fn take(&self) -> Option<Permit> {
        self.0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < LIMIT).then_some(count + 1)
            })
            .ok()
            .map(|_| Permit(self.0.clone()))
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}
