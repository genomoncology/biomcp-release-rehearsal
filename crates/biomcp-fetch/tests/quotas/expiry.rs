//! Resource boundary cases need controlled idle time without sleeping or network work.
use super::{IDLE_BUCKET_TTL, Limiter, Quota, Rate, SourceKey, TOKEN};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

static QUOTA: Quota = Quota::new(
    SourceKey::declare("expiry-test"),
    Rate::per_second(1),
    None,
    1,
);

#[test]
fn idle_expiry_waits_for_refill_and_retains_active_or_queued_calls() {
    let limiter = Limiter::new(QUOTA, false, "0.000001".parse().unwrap());
    let origin = Instant::now();
    {
        let mut state = limiter.state.lock().unwrap();
        state.updated = Some(origin);
        state.last_used = Some(origin);
    }
    let idle = origin + IDLE_BUCKET_TTL;
    assert!(
        !limiter.expired_at(idle, idle),
        "idle expiry must not reset spent scaled credit"
    );
    let refilled = origin + Duration::from_secs(1_000_000);
    assert!(
        limiter.expired_at(refilled, refilled),
        "an idle refilled bucket must expire"
    );
    {
        let mut state = limiter.state.lock().unwrap();
        state.in_flight = 1;
        state.credit = TOKEN;
    }
    assert!(
        !limiter.expired_at(refilled, refilled),
        "an active permit must retain its bucket"
    );
    {
        let mut state = limiter.state.lock().unwrap();
        state.in_flight = 0;
        state.queue.push_back(Arc::new(()));
    }
    assert!(
        !limiter.expired_at(refilled, refilled),
        "a queued caller must retain its bucket"
    );
}
