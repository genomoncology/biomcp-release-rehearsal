//! Resolve each source request's effective pacing identity once at admission.
use super::limiter::{Caller, Permit};
use crate::{KeyUse, Leg};
use std::time::Duration;

impl Leg<'_> {
    pub(crate) fn acquire_quota(&self) -> Result<Option<Permit<'_>>, Duration> {
        self.acquire_quota_by(self.budget())
    }
    /// Wait for a slot no longer than `deadline`, which may be shorter than the leg's budget.
    pub(crate) fn acquire_quota_by<'a>(
        &'a self,
        deadline: crate::Deadline<'a>,
    ) -> Result<Option<Permit<'a>>, Duration> {
        let caller = if self.fetch.hosted {
            Caller::Hosted(match self.spec.key {
                KeyUse::Optional(key) | KeyUse::Required(key) => {
                    self.fetch.secrets.fingerprint(key.secret)
                }
                KeyUse::None => None,
            })
        } else {
            Caller::Process
        };
        self.fetch.limiters.acquire(
            self.fetch.config.source_pacing.resolve(self.spec.quota),
            self.keyed(),
            caller,
            self.fetch.config.rate_share,
            deadline,
        )
    }
}
