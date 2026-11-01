//! Provider identities and validated published rate declarations.
use std::time::Duration;

/// A provider's declared limit identity, independent of URLs and source names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceKey(&'static str);

impl SourceKey {
    /// Stable configuration name of this provider bucket.
    pub const fn name(self) -> &'static str {
        self.0
    }

    /// Declare a nonempty key containing lowercase ASCII letters, digits and hyphens.
    ///
    /// # Panics
    /// Invalid declarations fail here, including during constant evaluation.
    pub const fn declare(name: &'static str) -> Self {
        assert!(!name.is_empty(), "empty source key");
        let mut bytes = name.as_bytes();
        while let [byte, rest @ ..] = bytes {
            assert!(
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-',
                "invalid source key"
            );
            bytes = rest;
        }
        Self(name)
    }
}

/// A validated token rate with an explicit burst allowance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rate {
    pub(super) per_second: u16,
    pub(super) seconds: u16,
    pub(super) burst: u16,
}

impl Rate {
    /// Declare a rate with no burst beyond the next single request.
    ///
    /// # Panics
    /// A zero rate fails during construction, including constant evaluation.
    pub const fn per_second(per_second: u16) -> Self {
        Self::new(per_second, 1)
    }

    /// Declare one request per whole-second interval, without a burst.
    ///
    /// # Panics
    /// A zero interval fails during construction, including constant evaluation.
    pub const fn every_seconds(seconds: u16) -> Self {
        assert!(seconds > 0, "zero rate interval");
        Self {
            per_second: 1,
            seconds,
            burst: 1,
        }
    }

    /// Declare evenly spaced requests over an explicit interval, without bursts.
    ///
    /// # Panics
    /// Zero requests or seconds fail during construction.
    pub const fn per_period(requests: u16, seconds: u16) -> Self {
        assert!(requests > 0 && seconds > 0, "zero rate or interval");
        Self {
            per_second: requests,
            seconds,
            burst: 1,
        }
    }
    /// Requests and seconds in the declared rate.
    pub const fn ratio(self) -> (u16, u16) {
        (self.per_second, self.seconds)
    }

    /// Declare a rate and burst allowance.
    ///
    /// # Panics
    /// Zero rates or bursts fail during construction, including constant evaluation.
    pub const fn new(per_second: u16, burst: u16) -> Self {
        assert!(per_second > 0 && burst > 0, "zero rate or burst");
        Self {
            per_second,
            seconds: 1,
            burst,
        }
    }
}

/// One provider limit shared by every source declaring its key.
#[derive(Clone, Copy, Debug)]
pub struct Quota {
    pub(super) key: SourceKey,
    pub(super) rate: Rate,
    pub(super) keyed_rate: Option<Rate>,
    pub(super) in_flight: u8,
    pub(super) refuse_wait: bool,
}

impl Quota {
    /// Provider bucket name accepted by source pacing configuration.
    pub const fn key(&self) -> SourceKey {
        self.key
    }
    /// Default and ceiling rate for requests without credentials.
    pub const fn rate(&self) -> Rate {
        self.rate
    }
    /// Separate default and ceiling for credentials, when declared.
    pub const fn keyed_rate(&self) -> Option<Rate> {
        self.keyed_rate
    }
    /// Conservative simultaneous request ceiling.
    pub const fn in_flight(&self) -> u8 {
        self.in_flight
    }
    /// Refuse immediately when the shared source spacing has not elapsed.
    pub const fn refuse_wait(mut self) -> Self {
        self.refuse_wait = true;
        self
    }
    /// Nominal request interval for this credential policy, excluding bursts.
    /// A diagnostic caller may use it to pace separate command processes.
    pub fn interval(&self, keyed: bool) -> Duration {
        let rate = self.credential_rate(keyed);
        Duration::from_nanos(
            (1_000_000_000 * u64::from(rate.seconds)).div_ceil(u64::from(rate.per_second)),
        )
    }
    pub(super) fn credential_rate(&self, keyed: bool) -> Rate {
        if keyed {
            self.keyed_rate.unwrap_or(self.rate)
        } else {
            self.rate
        }
    }
    /// Declare validated provider limits.
    ///
    /// # Panics
    /// A zero in-flight limit fails during construction, including constant evaluation.
    pub const fn new(key: SourceKey, rate: Rate, keyed_rate: Option<Rate>, in_flight: u8) -> Self {
        assert!(in_flight > 0, "zero in-flight limit");
        Self {
            key,
            rate,
            keyed_rate,
            in_flight,
            refuse_wait: false,
        }
    }
}
