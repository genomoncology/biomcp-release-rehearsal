//! Preserve native processing timestamps without inventing a missing timezone.
use serde::{Deserialize, Serialize};
mod date_time;
mod local;
mod updated;
mod value;
pub use date_time::{SourceDateTime, SourceDateTimeValue};
pub use local::SourceLocalTimestampValue;
pub use updated::SourceUpdated;
pub use value::SourceTimestampValue;

/// Checked source timestamp with its stated precision and offset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceTimestamp {
    /// Checked RFC 3339 source timestamp.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "date-time")))]
    pub value: SourceTimestampValue,
    /// Whether the native timestamp includes a timezone.
    pub timezone: SourceTimestampTimezone,
}

biodata::wire_enum! {
    /// Availability of timezone information in a native timestamp.
    SourceTimestampTimezone {
        /// The value retains the timezone supplied by the source.
        InValue => "in-value",
        /// The source supplied no timezone; none has been inferred.
        SourceGaveNone => "source-gave-none",
    }
}

impl std::str::FromStr for SourceTimestamp {
    type Err = biodata::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            value: value.parse()?,
            timezone: SourceTimestampTimezone::InValue,
        })
    }
}
