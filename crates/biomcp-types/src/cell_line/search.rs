//! Checked cell-line command input, separate from the unchanged literal-search reply query.
use super::query::{check_search_limit, default_limit};
use super::{CellLineSearch, CellLineVariationMatch};
use crate::QueryInvalid;
use crate::query::ProteinQuery;
use biodata::Text;
use serde::{Deserialize, Serialize};
/// One native cell line search with optional text, species, disease, variation and derived-line filters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CommandFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "CommandFields", extend(
    "anyOf" = [{"required":["query"]},{"required":["species"]},{"required":["disease"]},
    {"required":["gene"]},{"required":["mutation"]},{"required":["protein"]}]
)))]
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
    /// Cellosaurus variation gene, as the source writes it (for example KRAS).
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    gene: Option<Text>,
    /// Cellosaurus variation text, as the source writes it (for example p.Gly12Asp).
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    mutation: Option<Text>,
    /// Protein change, sent in the source's spelling (G12D is sent as p.Gly12Asp).
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "ProteinQuery"))]
    protein: Option<ProteinQuery>,
    /// Leave out lines derived from a parent line; needs another search term.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    exclude_derived: bool,
}
crate::query::validation::checked_query!(CellLineSearchCommand, CommandFields);
impl TryFrom<CommandFields> for CellLineSearchCommand {
    type Error = QueryInvalid;
    fn try_from(v: CommandFields) -> Result<Self, Self::Error> {
        check_search_limit(v.limit)?;
        if v.query.is_none()
            && v.species.is_none()
            && v.disease.is_none()
            && v.gene.is_none()
            && v.mutation.is_none()
            && v.protein.is_none()
        {
            return Err(QueryInvalid::new(&[
                "query", "species", "disease", "gene", "mutation", "protein",
            ]));
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
    /// Source sequence variation gene text.
    pub fn gene(&self) -> Option<&Text> {
        self.0.gene.as_ref()
    }
    /// Mutation terms in the source's spelling: literal text, then the parsed protein change.
    pub fn mutation_terms(&self) -> Vec<Text> {
        let protein = self.0.protein.as_ref().and_then(|protein| {
            let [.., source] = protein.forms();
            Text::new(&source).ok()
        });
        self.0.mutation.iter().cloned().chain(protein).collect()
    }
    /// Whether the source query leaves out lines with a recorded parent.
    pub fn exclude_derived(&self) -> bool {
        self.0.exclude_derived
    }
    /// What a variation filter asked the source to match, if any.
    pub fn variation_match(&self) -> Option<CellLineVariationMatch> {
        let terms: Vec<Text> = self
            .gene()
            .cloned()
            .into_iter()
            .chain(self.mutation_terms())
            .collect();
        (!terms.is_empty()).then(|| CellLineVariationMatch::new(terms))
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
        if self.species().is_some()
            || self.disease().is_some()
            || self.variation_match().is_some()
            || self.exclude_derived()
        {
            return None;
        }
        self.query()
            .map(|query| CellLineSearch::from_parts(query.clone(), self.limit(), self.offset()))
    }
}
