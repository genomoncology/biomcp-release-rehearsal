//! Checked native local timestamp text without an inferred offset.
use std::{fmt, str::FromStr};

/// Native calendar and clock text with no stated timezone.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceLocalTimestampValue(Box<str>);

impl SourceLocalTimestampValue {
    /// Borrow the unchanged native local timestamp.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl FromStr for SourceLocalTimestampValue {
    type Err = biodata::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if !value.is_ascii()
            || value.len() < 19
            || !matches!(value.as_bytes().get(10), Some(b'T' | b't'))
        {
            return Err(biodata::Error::DateInvalid);
        }
        value[..10]
            .parse::<biodata::PartialDate>()
            .map_err(|_| biodata::Error::DateInvalid)?;
        super::value::source_clock(&value[11..])?;
        Ok(Self(value.into()))
    }
}
impl fmt::Display for SourceLocalTimestampValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
biodata::text_value!(
    SourceLocalTimestampValue,
    "SourceLocalTimestampValue",
    concat!(
        r"^(?!0000)[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])[Tt]",
        r"([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(?![\s\S])"
    ),
    concat!(
        "Native source local timestamp preserving fractional precision without a timezone. ",
        "The parser additionally validates the calendar day; no UTC leap-second position is inferred."
    )
);
