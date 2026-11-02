//! RFC 3339 sections 5.6–5.7 validation without normalizing source text.
use std::{fmt, str::FromStr};

/// Checked RFC 3339 text preserving the source's offset and fractional precision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceTimestampValue(Box<str>);

impl SourceTimestampValue {
    /// Borrow the exact checked source text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for SourceTimestampValue {
    type Err = biodata::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid = || biodata::Error::DateInvalid;
        let bytes = value.as_bytes();
        if !value.is_ascii() || bytes.len() < 20 || !matches!(bytes.get(10), Some(b'T' | b't')) {
            return Err(invalid());
        }
        let date = value[..10]
            .parse::<biodata::PartialDate>()
            .map_err(|_| biodata::Error::DateInvalid)?;
        let (clock, offset) = match bytes.last() {
            Some(b'Z' | b'z') => (&value[11..value.len() - 1], 0),
            _ => {
                let start = value.len() - 6;
                let sign = match bytes.get(start) {
                    Some(b'+') => 1,
                    Some(b'-') => -1,
                    _ => return Err(invalid()),
                };
                let zone = &value[start + 1..];
                if zone.as_bytes().get(2) != Some(&b':') {
                    return Err(invalid());
                }
                let hours = pair(&zone[..2], 23)?;
                let minutes = pair(&zone[3..], 59)?;
                (&value[11..start], sign * (hours * 60 + minutes))
            }
        };
        let (hour, minute, second) = source_clock(clock)?;
        if second == 60 && !month_end_leap_minute(date, hour * 60 + minute - offset) {
            return Err(invalid());
        }
        Ok(Self(value.into()))
    }
}

pub(super) fn source_clock(clock: &str) -> Result<(i16, i16, i16), biodata::Error> {
    let (clock, fraction) = clock
        .split_once('.')
        .map_or((clock, None), |(c, f)| (c, Some(f)));
    if clock.len() != 8
        || clock.as_bytes().get(2) != Some(&b':')
        || clock.as_bytes().get(5) != Some(&b':')
        || fraction.is_some_and(|f| f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(biodata::Error::DateInvalid);
    }
    let hour = pair(&clock[..2], 23)?;
    let minute = pair(&clock[3..5], 59)?;
    let second = pair(&clock[6..], 60)?;
    Ok((hour, minute, second))
}

fn pair(value: &str, maximum: i16) -> Result<i16, biodata::Error> {
    if value.len() == 2
        && value.bytes().all(|b| b.is_ascii_digit())
        && let Ok(number) = value.parse::<i16>()
        && number <= maximum
    {
        return Ok(number);
    }
    Err(biodata::Error::DateInvalid)
}

// RFC 3339 places inserted seconds at a UTC month's final minute. This checks
// that position, without predicting or maintaining future leap announcements.
fn month_end_leap_minute(date: biodata::PartialDate, utc_minute: i16) -> bool {
    let Some(day) = date.day() else { return false };
    match utc_minute {
        -1 => day == 1,
        1439 => biodata::PartialDate::new(date.year(), date.month(), Some(day + 1)).is_err(),
        _ => false,
    }
}

impl fmt::Display for SourceTimestampValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

biodata::text_value!(
    SourceTimestampValue,
    "SourceTimestampValue",
    concat!(
        r"^(?!0000)[0-9]{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])[Tt]",
        r"([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?",
        r"([Zz]|[+-]([01][0-9]|2[0-3]):[0-5][0-9])(?![\s\S])"
    ),
    concat!(
        "RFC 3339 source text preserving offset and precision. ",
        "The parser additionally validates the calendar day and UTC month-end position of leap seconds. ",
        "Years follow PartialDate's 0001–9999 range."
    )
);
