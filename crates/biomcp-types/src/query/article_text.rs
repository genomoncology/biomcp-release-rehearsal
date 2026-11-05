use super::gene::{QueryInvalid, present};
use crate::{ArticleInput, Text};
use serde::{Deserialize, Serialize};
/// Read one bounded body part, find matching rows, or save the complete Markdown document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature="schema",
 schemars(with="Fields",
 extend("anyOf"=[{"required":["part"]},
{"required":["find"]},
{"required":["variant"]},
{"required":["save"]}])))]
pub struct ArticleText(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// PubMed or PMC identity. DOI selectors are not accepted by the reader.
    #[cfg_attr(feature="schema", schemars(extend("x-positional"=0)))]
    input: ArticleInput,
    /// One-based outline part number.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1)))]
    part: Option<u32>,
    /// Case-insensitive literal text to find.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    find: Option<Text>,
    /// Explicit checked protein substitution. Example: --variant p.V600E.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    variant: Option<Text>,
    /// Save the whole original document to an explicit file or existing folder. Example: --save article-output.
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
    /// Number of matching rows already returned.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(ArticleText, Fields);
impl TryFrom<Fields> for ArticleText {
    type Error = QueryInvalid;
    fn try_from(v: Fields) -> Result<Self, QueryInvalid> {
        if matches!(v.input, ArticleInput::Doi(_)) {
            return Err(QueryInvalid::new(&["input"]));
        }
        if [
            v.part.is_some(),
            v.find.is_some(),
            v.variant.is_some(),
            v.save.is_some(),
        ]
        .into_iter()
        .filter(|v| *v)
        .count()
            != 1
        {
            return Err(QueryInvalid::new(&["part", "find", "variant", "save"]));
        }
        if v.part == Some(0) {
            return Err(QueryInvalid::new(&["part"]));
        }
        if (v.part.is_some() || v.save.is_some()) && v.offset != 0 {
            return Err(QueryInvalid::new(&["part", "save", "offset"]));
        }
        if v.replace && v.save.is_none() {
            return Err(QueryInvalid::new(&["replace", "save"]));
        }
        Ok(Self(v))
    }
}
impl ArticleText {
    /// Build a part selector used in source continuation actions.
    pub fn part(input: ArticleInput, part: u32) -> Result<Self, QueryInvalid> {
        let mut fields = Fields::new(input);
        fields.part = Some(part);
        fields.try_into()
    }
    /// Build a checked literal selector used in source follow-up actions.
    pub fn finding(input: ArticleInput, find: &str) -> Result<Self, QueryInvalid> {
        let mut fields = Fields::new(input);
        fields.find = Some(Text::new(find).map_err(|_| QueryInvalid::new(&["find"]))?);
        fields.try_into()
    }
    /// Explicit save destination, absent for ordinary reader views.
    pub fn save(&self) -> Option<&Text> {
        self.0.save.as_ref()
    }
    /// Whether replacement was explicitly requested.
    pub fn replace(&self) -> bool {
        self.0.replace
    }
    /// Original checked identity.
    pub fn input(&self) -> &ArticleInput {
        &self.0.input
    }
    /// Selected part.
    pub fn selected_part(&self) -> Option<u32> {
        self.0.part
    }
    /// Literal search term.
    pub fn find(&self) -> Option<&Text> {
        self.0.find.as_ref()
    }
    /// Explicit variant selector; the reader validates its private notation before I/O.
    pub fn variant(&self) -> Option<&Text> {
        self.0.variant.as_ref()
    }
    /// Checked hit offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Continue a find page without changing its selector.
    pub fn page(&self, offset: u32) -> Self {
        let mut v = self.clone();
        v.0.offset = offset;
        v
    }
}
impl Fields {
    fn new(input: ArticleInput) -> Self {
        Self {
            input,
            part: None,
            find: None,
            variant: None,
            offset: 0,
            save: None,
            replace: false,
        }
    }
}
