//! Checked cell-line command input, separate from the unchanged literal-search reply query.
use super::CellLineSearch;
use super::query::{check_search_limit, default_limit};
use crate::QueryInvalid;
use biodata::Text;
use serde::{Deserialize, Serialize};
/// One native cell line search with optional text, species and NCIt disease filters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CommandFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "CommandFields", extend("anyOf" = [{"required":["query"]},{"required":["species"]},{"required":["disease"]}])))]
pub struct CellLineSearchCommand(CommandFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct CommandFields {
    /// Literal source query from the positional argument.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text", extend("x-positional" = 0)))]
    query: Option<Text>,
    /// Native page size.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 25)))]
    limit: u32,
    /// Native start offset.
    #[serde(default)]
    offset: u32,
    /// Cellosaurus species: NCBI Taxonomy name or ID (for example Homo sapiens or 9606).
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    species: Option<Text>,
    /// Cellosaurus disease: NCIt name or ID (for example Melanoma or C3224).
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    disease: Option<Text>,
}
crate::query::validation::checked_query!(CellLineSearchCommand, CommandFields);
impl TryFrom<CommandFields> for CellLineSearchCommand {
    type Error = QueryInvalid;
    fn try_from(v: CommandFields) -> Result<Self, Self::Error> {
        check_search_limit(v.limit)?;
        if v.query.is_none() && v.species.is_none() && v.disease.is_none() {
            return Err(QueryInvalid::new(&["query", "species", "disease"]));
        }
        Ok(Self(v))
    }
}
impl CellLineSearchCommand {
    /// Literal checked search text.
    pub fn query(&self) -> Option<&Text> {
        self.0.query.as_ref()
    }
    /// Native species name or taxonomy identifier.
    pub fn species(&self) -> Option<&Text> {
        self.0.species.as_ref()
    }
    /// Native NCIt disease name or identifier.
    pub fn disease(&self) -> Option<&Text> {
        self.0.disease.as_ref()
    }
    /// Native page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native start offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for CellLineSearchCommand {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

impl CellLineSearchCommand {
    /// Keep the original response contract for commands without filters.
    pub fn unfiltered(&self) -> Option<CellLineSearch> {
        if self.species().is_some() || self.disease().is_some() {
            return None;
        }
        self.query()
            .map(|query| CellLineSearch::from_parts(query.clone(), self.limit(), self.offset()))
    }
}
