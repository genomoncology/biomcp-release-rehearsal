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
pub(super) struct Slots {
    count: Arc<AtomicUsize>,
    limit: usize,
}
#[expect(
    clippy::disallowed_types,
    reason = "Release one HTTP admission count when its worker leaves the phase"
)]
pub(super) struct Permit(Arc<AtomicUsize>);
impl Slots {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            count: Arc::default(),
            limit,
        }
    }
    pub(super) fn take(&self) -> Option<Permit> {
        self.count
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < self.limit).then_some(count + 1)
            })
            .ok()
            .map(|_| Permit(self.count.clone()))
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}
