//! Exact operator rates, validated against each source's declared ceiling.
use super::{Quota, Rate, SourceKey};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Requests evenly spaced over seconds, without a burst allowance.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PacingRate {
    /// Positive requests per interval.
    pub requests: u16,
    /// Positive interval in seconds.
    pub seconds: u16,
}
impl PacingRate {
    fn checked(self, ceiling: Rate) -> Option<Rate> {
        let (requests, seconds) = ceiling.ratio();
        // Cross products fit u32 even for the largest permitted u16 components.
        (self.requests > 0
            && self.seconds > 0
            && u32::from(self.requests) * u32::from(seconds)
                <= u32::from(requests) * u32::from(self.seconds))
        .then(|| Rate::per_period(self.requests, self.seconds))
    }
}
/// Each supplied field overrides its file value; omitted fields remain independent.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PacingOverride {
    /// Requests without credentials; does not change a separate keyed rate.
    pub rate: Option<PacingRate>,
    /// Requests with credentials, only for a source with a distinct keyed policy.
    pub keyed_rate: Option<PacingRate>,
    /// Positive simultaneous request count.
    pub in_flight: Option<u8>,
}
impl PacingOverride {
    /// Merge explicitly supplied flag fields over file fields.
    pub fn merge(self, flags: Self) -> Self {
        Self {
            rate: flags.rate.or(self.rate),
            keyed_rate: flags.keyed_rate.or(self.keyed_rate),
            in_flight: flags.in_flight.or(self.in_flight),
        }
    }
}
/// Validated transport values; construction cannot raise a declared ceiling.
#[derive(Debug, Default)]
pub struct SourcePacing(BTreeMap<SourceKey, Quota>);
impl SourcePacing {
    /// Check a named registry entry and retain its effective transport values.
    /// Errors name the rejected field and never contain operator values.
    pub fn insert(&mut self, ceiling: &Quota, input: PacingOverride) -> Result<(), &'static str> {
        let mut effective = *ceiling;
        if let Some(rate) = input.rate {
            effective.rate = rate.checked(ceiling.rate).ok_or("rate")?;
        }
        if let Some(rate) = input.keyed_rate {
            effective.keyed_rate = Some(
                rate.checked(ceiling.keyed_rate.ok_or("keyed_rate")?)
                    .ok_or("keyed_rate")?,
            );
        }
        if let Some(count) = input.in_flight {
            if count == 0 || count > ceiling.in_flight {
                return Err("in_flight");
            }
            effective.in_flight = count;
        }
        self.0.insert(ceiling.key, effective);
        Ok(())
    }
    /// Apply a checked override or retain the provider declaration.
    pub(super) fn resolve(&self, quota: &Quota) -> Quota {
        self.0.get(&quota.key).copied().unwrap_or(*quota)
    }
}
