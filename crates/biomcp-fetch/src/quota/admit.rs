//! Resolve each source request's effective pacing identity once at admission.
use super::limiter::{Caller, Permit};
use crate::{KeyUse, Leg};
use std::time::Duration;

impl Leg<'_> {
    pub(crate) fn acquire_quota(&self) -> Result<Option<Permit<'_>>, Duration> {
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
            self.budget(),
        )
    }
}
