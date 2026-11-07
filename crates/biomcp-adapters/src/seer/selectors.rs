use biodata::Text;
use std::fmt;
/// Native SEER selector tables reviewed against the retained publisher catalog.
/// Catalog output remains source-driven; these tables validate input before I/O.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selector {
    /// Native sex group.
    Sex,
    /// Native race and ethnicity group.
    Race,
    /// Native age group.
    AgeRange,
    /// Requested data model.
    Dataset,
    /// Incidence year range.
    Period,
    /// Relative survival interval.
    Interval,
}
impl Selector {
    /// Command option owning this selector.
    pub const fn flag(self) -> &'static str {
        match self {
            Self::Sex => "--sex",
            Self::Race => "--race",
            Self::AgeRange => "--age-range",
            Self::Dataset => "--dataset",
            Self::Period => "--period",
            Self::Interval => "--interval",
        }
    }
    /// Reviewed native codes in their captured catalog order.
    pub const fn valid_codes(self) -> &'static [&'static str] {
        match self {
            Self::Sex => &["1", "3", "2"],
            Self::Race => &["1", "6", "5", "4", "9", "8", "3", "2"],
            Self::AgeRange => &[
                "1", "9", "6", "141", "157", "16", "11", "62", "122", "160", "166", "15", "200",
                "201", "202", "203", "204", "205", "206", "207", "208", "209", "210", "211", "212",
                "213", "214", "215", "216", "217", "218", "219", "300", "301", "302", "303", "304",
                "305", "306", "307", "308", "309", "310", "311", "312", "313", "314", "315", "316",
                "317", "318", "319", "320", "999",
            ],
            Self::Dataset => &["3", "1", "4"],
            Self::Period => &[
                "31", "21", "1", "23", "13", "3", "33", "14", "15", "35", "36", "16", "4", "9",
                "5", "25", "8", "28", "12",
            ],
            Self::Interval => &["1", "2", "3", "5", "10"],
        }
    }
    /// Resolve only published all-group demographic defaults; other omissions fail.
    pub fn resolve(self, supplied: Option<&Text>) -> Result<Text, SelectorError> {
        let code = supplied.map(Text::as_str).or(match self {
            Self::Sex | Self::Race | Self::AgeRange => Some("1"),
            Self::Dataset | Self::Period | Self::Interval => None,
        });
        let code = code
            .filter(|code| self.valid_codes().contains(code))
            .ok_or(SelectorError {
                selectors: vec![self],
            })?;
        code.parse().map_err(|_| SelectorError {
            selectors: vec![self],
        })
    }
}
/// A missing or unknown source selector, without retaining user input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorError {
    /// Native selector whose required or valid value is missing.
    pub selectors: Vec<Selector>,
}
impl fmt::Display for SelectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("seer.selector: ")?;
        for (index, selector) in self.selectors.iter().enumerate() {
            if index != 0 {
                f.write_str("; ")?;
            }
            write!(
                f,
                "{} requires one of {}",
                selector.flag(),
                selector.valid_codes().join(", ")
            )?;
        }
        Ok(())
    }
}
impl std::error::Error for SelectorError {}
