//! Process-owned FIFO source buckets and their admission permits.
use super::{Quota, Rate, RateShare, SourceKey};
use crate::Deadline;
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded transport state across the request owners named by this module"
)]
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BinaryHeap, VecDeque},
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Default)]
#[expect(
    clippy::disallowed_types,
    reason = "Share source quota counters and FIFO waits within one Fetch"
)]
pub(crate) struct Limiters(Mutex<BTreeMap<(SourceKey, Caller), Arc<Limiter>>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Caller {
    Process,
    Hosted(Option<crate::KeyFingerprint>),
}
const IDLE_BUCKET_TTL: Duration = Duration::from_secs(300);
#[cfg(test)]
#[path = "../../tests/quotas/expiry.rs"]
mod tests;

impl Limiters {
    pub(super) fn acquire<'a>(
        &self,
        quota: Quota,
        keyed: bool,
        caller: Caller,
        share: RateShare,
        deadline: Deadline<'a>,
    ) -> Result<Option<Permit<'a>>, Duration> {
        let mut buckets = self.0.lock().unwrap_or_else(|e| e.into_inner());
        buckets.retain(|(_, caller), limiter| {
            matches!(caller, Caller::Process)
                || Arc::strong_count(limiter) > 1
                || !limiter.expired_at(Instant::now(), deadline.now())
        });
        let share = if matches!(caller, Caller::Hosted(Some(_))) {
            share
        } else {
            RateShare::default()
        };
        let limiter = buckets
            .entry((quota.key, caller))
            .or_insert_with(|| Arc::new(Limiter::new(quota, keyed, share)))
            .clone();
        drop(buckets);
        limiter.acquire(keyed, deadline)
    }
}

#[derive(Debug)]
#[expect(
    clippy::disallowed_types,
    reason = "Share source quota counters and FIFO waits within one Fetch"
)]
struct Limiter {
    quota: Quota,
    share: RateShare,
    expiry_rate: Rate,
    state: Mutex<State>,
    wake: Condvar,
}

#[derive(Debug, Default)]
struct State {
    queue: VecDeque<Arc<()>>,
    in_flight: u8,
    updated: Option<Instant>,
    credit: u128,
    completed: Option<BinaryHeap<Reverse<Instant>>>,
    last_used: Option<Instant>,
}

// Nanosecond token units retain exact rates across credential changes.
const TOKEN: u128 = 1_000_000_000;

impl State {
    fn admission_time(&self, deadline: Deadline<'_>) -> Instant {
        let now = deadline.now();
        let slot = self
            .completed
            .as_ref()
            .and_then(|times| times.peek())
            .map(|time| time.0);
        if deadline.virtual_time()
            && let Some(last) = self
                .updated
                .into_iter()
                .chain(slot)
                .max()
                .filter(|last| *last > now)
        {
            deadline.advance(last.duration_since(now));
        }
        deadline.now()
    }

    fn available(&self, rate: Rate, share: RateShare, now: Instant) -> u128 {
        let capacity = u128::from(rate.burst) * TOKEN;
        self.updated
            .map(|last| {
                self.credit
                    .saturating_add(share.gained(now.saturating_duration_since(last), rate))
                    .min(capacity)
            })
            .unwrap_or(capacity)
    }
}

impl Limiter {
    #[expect(
        clippy::disallowed_types,
        reason = "Share source quota counters and FIFO waits within one Fetch"
    )]
    fn new(quota: Quota, keyed: bool, share: RateShare) -> Self {
        Self {
            quota,
            share,
            expiry_rate: quota.credential_rate(keyed),
            state: Mutex::new(State::default()),
            wake: Condvar::new(),
        }
    }

    fn expired_at(&self, wall: Instant, clock: Instant) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.in_flight == 0
            && state.queue.is_empty()
            && state
                .last_used
                .is_some_and(|last| wall.saturating_duration_since(last) >= IDLE_BUCKET_TTL)
            && state.available(self.expiry_rate, self.share, clock)
                == u128::from(self.expiry_rate.burst) * TOKEN
    }

    fn acquire<'a>(
        self: Arc<Self>,
        keyed: bool,
        deadline: Deadline<'a>,
    ) -> Result<Option<Permit<'a>>, Duration> {
        let rate = self.quota.credential_rate(keyed);
        let ticket = Arc::new(());
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.last_used = Some(Instant::now());
        if deadline.virtual_time() && state.completed.is_none() {
            state.completed = Some(
                (0..self.quota.in_flight)
                    .map(|_| Reverse(deadline.now()))
                    .collect(),
            );
        }
        state.queue.push_back(ticket.clone());
        loop {
            if deadline.expired() {
                state.queue.retain(|v| !Arc::ptr_eq(v, &ticket));
                self.wake.notify_all();
                return Ok(None);
            }
            let now = state.admission_time(deadline);
            if deadline.expired() {
                continue;
            }
            let available = state.available(rate, self.share, now);
            let delay = self.share.delay(TOKEN.saturating_sub(available), rate);
            let head = state.queue.front().is_some_and(|v| Arc::ptr_eq(v, &ticket));
            if head && state.in_flight < self.quota.in_flight && delay.is_zero() {
                state.queue.pop_front();
                state.in_flight += 1;
                if let Some(completed) = &mut state.completed {
                    completed.pop();
                }
                state.credit = available - TOKEN;
                state.updated = Some(now);
                self.wake.notify_all();
                drop(state);
                return Ok(Some(Permit {
                    limiter: self,
                    deadline,
                }));
            }
            if self.quota.refuse_wait {
                state.queue.retain(|v| !Arc::ptr_eq(v, &ticket));
                self.wake.notify_all();
                return Err(delay);
            }
            let wait = if head && state.in_flight < self.quota.in_flight {
                deadline.remaining().min(delay)
            } else {
                deadline.remaining()
            };
            if deadline.virtual_time() && head && state.in_flight < self.quota.in_flight {
                deadline.advance(wait);
                continue;
            }
            if deadline.virtual_time() {
                state = self.wake.wait(state).unwrap_or_else(|e| e.into_inner());
                continue;
            }
            // Condvar releases the mutex while this caller waits for its rate slot or predecessor.
            state = self
                .wake
                .wait_timeout(state, deadline.wait_slice(wait))
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }
}

#[derive(Debug)]
pub(crate) struct Permit<'a> {
    limiter: Arc<Limiter>,
    deadline: Deadline<'a>,
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        let mut state = self.limiter.state.lock().unwrap_or_else(|e| e.into_inner());
        state.in_flight -= 1;
        state.last_used = Some(Instant::now());
        if let Some(completed) = &mut state.completed {
            completed.push(Reverse(self.deadline.now()));
        }
        self.limiter.wake.notify_all();
    }
}
