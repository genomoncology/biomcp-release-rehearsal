//! Construct source state and reason together, while exposing an immutable wire view.
use crate::{Reason, SourceState};
use serde::{Deserialize, Serialize};

/// A source result that cannot attach a failure reason to a current answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceStatus {
    /// Checked current data answered without a failure.
    Ok,
    /// Checked stale data answered, optionally retaining its fallback reason.
    Stale(Option<Reason>),
    /// Another process is synchronizing the source.
    Syncing,
    /// The request failed, optionally retaining its provider reason.
    Failed(Option<Reason>),
    /// The source request was skipped, optionally retaining its reason.
    Skipped(Option<Reason>),
}

/// Read-only state and reason of a source outcome.
///
/// A copied view cannot be installed into an outcome; updates take `SourceStatus`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceStatusView {
    /// Retained state.
    pub state: SourceState,
    /// Failure or stale-fallback reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Reason>,
}

impl From<SourceStatus> for SourceStatusView {
    fn from(status: SourceStatus) -> Self {
        let (state, reason) = match status {
            SourceStatus::Ok => (SourceState::Ok, None),
            SourceStatus::Stale(reason) => (SourceState::Stale, reason),
            SourceStatus::Syncing => (SourceState::Syncing, Some(Reason::Syncing)),
            SourceStatus::Failed(reason) => (SourceState::Failed, reason),
            SourceStatus::Skipped(reason) => (SourceState::Skipped, reason),
        };
        Self { state, reason }
    }
}

impl SourceStatusView {
    pub(super) fn status(&self) -> SourceStatus {
        match self.state {
            SourceState::Ok => SourceStatus::Ok,
            SourceState::Stale => SourceStatus::Stale(self.reason),
            SourceState::Syncing => SourceStatus::Syncing,
            SourceState::Failed => SourceStatus::Failed(self.reason),
            SourceState::Skipped => SourceStatus::Skipped(self.reason),
        }
    }
}

impl<'de> Deserialize<'de> for SourceStatusView {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            state: SourceState,
            reason: Option<Reason>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let status = match (wire.state, wire.reason) {
            (SourceState::Ok, None) => SourceStatus::Ok,
            (SourceState::Ok, Some(_)) => {
                return Err(serde::de::Error::custom(
                    "current source answer cannot carry a failure reason",
                ));
            }
            (SourceState::Stale, reason) => SourceStatus::Stale(reason),
            (SourceState::Syncing, None | Some(Reason::Syncing)) => SourceStatus::Syncing,
            (SourceState::Syncing, Some(_)) => {
                return Err(serde::de::Error::custom(
                    "syncing source requires the syncing reason",
                ));
            }
            (SourceState::Failed, reason) => SourceStatus::Failed(reason),
            (SourceState::Skipped, reason) => SourceStatus::Skipped(reason),
        };
        Ok(status.into())
    }
}
