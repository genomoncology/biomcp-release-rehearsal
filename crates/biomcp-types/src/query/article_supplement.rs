use super::gene::{QueryInvalid, present};
use crate::{ArticleInput, Error, Text, text_value};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
/// Inclusive original one-based row range, at most one page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetRows(u32, u32);
impl FromStr for SheetRows {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let (a, b) = s.split_once('-').ok_or(Error::ArticleIdInvalid)?;
        if [a, b]
            .iter()
            .any(|v| v.starts_with('0') || !v.bytes().all(|c| c.is_ascii_digit()))
        {
            return Err(Error::ArticleIdInvalid);
        }
        let a: u32 = a.parse().map_err(|_| Error::ArticleIdInvalid)?;
        let b: u32 = b.parse().map_err(|_| Error::ArticleIdInvalid)?;
        if a == 0 || b < a || b - a >= 200 {
            return Err(Error::ArticleIdInvalid);
        }
        Ok(Self(a, b))
    }
}
impl SheetRows {
    /// Inclusive first original row number.
    pub fn start(&self) -> u32 {
        self.0
    }
    /// Inclusive last original row number.
    pub fn end(&self) -> u32 {
        self.1
    }
}
impl fmt::Display for SheetRows {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.0, self.1)
    }
}
text_value!(
    SheetRows,
    "SheetRows",
    format!(
        "^{}-{}$",
        biodata::schema_pattern::decimal(u32::MAX.into()),
        biodata::schema_pattern::decimal(u32::MAX.into())
    ),
    "Inclusive original one-based row range. The parser additionally requires the end to be at least the start and fewer than 200 rows after it."
);
/// Read a checked supplement file or page its original rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct ArticleSupplement(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// PubMed or PMC identity.
    #[cfg_attr(feature="schema", schemars(extend("x-positional"=0)))]
    input: ArticleInput,
    /// Exact file name from the supplements section.
    #[cfg_attr(feature="schema", schemars(extend("x-positional"=1)))]
    file: Text,
    /// Inclusive original row range.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "SheetRows"))]
    rows: Option<SheetRows>,
    /// Sheet name for row selection.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    sheet: Option<Text>,
    /// Literal case-insensitive needle.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    find: Option<Text>,
    /// Explicit protein substitution.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    variant: Option<Text>,
    /// Save the whole original document to an explicit file or existing folder.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    save: Option<Text>,
    /// Replace an existing destination only when explicitly requested with save.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    replace: bool,
    /// Matching-row or text-fragment offset.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(ArticleSupplement, Fields);
impl TryFrom<Fields> for ArticleSupplement {
    type Error = QueryInvalid;
    fn try_from(v: Fields) -> Result<Self, QueryInvalid> {
        if matches!(v.input, ArticleInput::Doi(_)) {
            return Err(QueryInvalid::new(&["input"]));
        }
        if [
            v.rows.is_some(),
            v.find.is_some(),
            v.variant.is_some(),
            v.save.is_some(),
        ]
        .into_iter()
        .filter(|v| *v)
        .count()
            > 1
        {
            return Err(QueryInvalid::new(&["rows", "find", "variant", "save"]));
        }
        if v.file.as_str().contains(['/', '\\']) {
            return Err(QueryInvalid::new(&["file"]));
        }
        if matches!(v.file.as_str(), "." | "..") {
            return Err(QueryInvalid::new(&["file"]));
        }
        if v.sheet.is_some() && v.rows.is_none() {
            return Err(QueryInvalid::new(&["sheet", "rows"]));
        }
        if v.save.is_some() && v.offset != 0 {
            return Err(QueryInvalid::new(&["save", "offset"]));
        }
        if v.replace && v.save.is_none() {
            return Err(QueryInvalid::new(&["replace", "save"]));
        }
        Ok(Self(v))
    }
}
impl ArticleSupplement {
    /// Build the initial file action.
    pub fn new(input: ArticleInput, file: Text) -> Result<Self, QueryInvalid> {
        Fields {
            input,
            file,
            rows: None,
            sheet: None,
            find: None,
            variant: None,
            offset: 0,
            save: None,
            replace: false,
        }
        .try_into()
    }
    /// Explicit save destination, absent for ordinary reader views.
    pub fn save(&self) -> Option<&Text> {
        self.0.save.as_ref()
    }
    /// Whether replacement was explicitly requested.
    pub fn replace(&self) -> bool {
        self.0.replace
    }
    /// Original identity.
    pub fn input(&self) -> &ArticleInput {
        &self.0.input
    }
    /// Exact name.
    pub fn file(&self) -> &Text {
        &self.0.file
    }
    /// Row range.
    pub fn rows(&self) -> Option<&SheetRows> {
        self.0.rows.as_ref()
    }
    /// Optional sheet.
    pub fn sheet(&self) -> Option<&Text> {
        self.0.sheet.as_ref()
    }
    /// Literal case-insensitive selector.
    pub fn find(&self) -> Option<&Text> {
        self.0.find.as_ref()
    }
    /// Explicit variant selector; the reader validates its private notation before I/O.
    pub fn variant(&self) -> Option<&Text> {
        self.0.variant.as_ref()
    }
    /// Result offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Continue a bounded page.
    pub fn page(&self, offset: u32) -> Self {
        let mut v = self.clone();
        v.0.offset = offset;
        v
    }
    /// Select the first bounded rows of a sheet.
    pub fn first_rows(&self, sheet: Text, last: u32) -> Self {
        let mut v = self.clone();
        v.0.find = None;
        v.0.variant = None;
        v.0.sheet = Some(sheet);
        v.0.rows = Some(SheetRows(1, last.clamp(1, 20)));
        v.0.offset = 0;
        v
    }
}
