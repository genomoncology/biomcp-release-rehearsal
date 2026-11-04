//! A source update retains either a date or a timestamp without adding precision.
use super::SourceTimestamp;
use serde::{Deserialize, Serialize};

/// Source update metadata preserving the date or timestamp supplied by its publisher.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SourceUpdated {
    /// A source date keeps its year, month or day precision.
    Date(biodata::PartialDate),
    /// A source timestamp keeps its stated fractional precision and offset.
    Timestamp(SourceTimestamp),
}
impl From<biodata::PartialDate> for SourceUpdated {
    fn from(date: biodata::PartialDate) -> Self {
        Self::Date(date)
    }
}
impl From<SourceTimestamp> for SourceUpdated {
    fn from(timestamp: SourceTimestamp) -> Self {
        Self::Timestamp(timestamp)
    }
}
