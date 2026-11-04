use crate::{Cytoband, GeneInput, LocusGroup, Text};
use serde::{Deserialize, Serialize};
const MIN_INPUTS: usize = 1;
const MAX_INPUTS: usize = 20;
const MIN_LIMIT: u32 = 1;
const MAX_LIMIT: u32 = 50;

pub use super::validation::QueryInvalid;

/// Get one to twenty human gene inputs in their original order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct GeneGet(GetFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// Gene symbols or checked identifiers, in input order.
    #[cfg_attr(feature = "schema", schemars(length(min = MIN_INPUTS, max = MAX_INPUTS), extend("x-positional" = 0)))]
    input: Box<[GeneInput]>,
    /// Requested section words, preserving caller order and repetitions.
    /// Trailing section words select sections. Use --section WORD to keep inputs that equal a section word as inputs.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(extend("x-trailing-section" = true)))]
    section: Box<[GeneSectionWord]>,
}
crate::query::validation::checked_query!(GeneGet, GetFields);
impl TryFrom<GetFields> for GeneGet {
    type Error = QueryInvalid;
    fn try_from(fields: GetFields) -> Result<Self, Self::Error> {
        Self::with_sections(fields.input, fields.section)
    }
}
impl GeneGet {
    /// Refuse an empty or oversized input batch.
    pub fn new(input: Box<[GeneInput]>) -> Result<Self, QueryInvalid> {
        Self::with_sections(input, Box::default())
    }
    /// Check inputs and retain the requested section words.
    pub fn with_sections(
        input: Box<[GeneInput]>,
        section: Box<[GeneSectionWord]>,
    ) -> Result<Self, QueryInvalid> {
        if !(MIN_INPUTS..=MAX_INPUTS).contains(&input.len()) {
            return Err(QueryInvalid::new(&["input"]));
        }
        Ok(Self(GetFields { input, section }))
    }
    /// Section words in caller order.
    pub fn sections(&self) -> &[GeneSectionWord] {
        &self.0.section
    }
    /// Inputs in caller order.
    pub fn input(&self) -> &[GeneInput] {
        &self.0.input
    }
}
/// Search genes by text, cytoband, or their intersection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields", extend("anyOf" = [{"required":["query"]},{"required":["band"]}])))]
pub struct GeneSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Words matched against symbols, previous symbols, aliases and names.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text", extend("x-positional" = 0)))]
    query: Option<Text>,
    /// A cytoband prefix.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Cytoband"))]
    band: Option<Cytoband>,
    /// Optional HGNC locus group intersected with the selector.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "LocusGroup"))]
    locus_group: Option<LocusGroup>,
    /// Number of results per page.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = MIN_LIMIT, max = MAX_LIMIT)))]
    limit: u32,
    /// Number of results to skip.
    #[serde(default)]
    offset: u32,
}
fn default_limit() -> u32 {
    10
}
crate::query::validation::checked_query!(GeneSearch, SearchFields);
impl TryFrom<SearchFields> for GeneSearch {
    type Error = QueryInvalid;
    fn try_from(fields: SearchFields) -> Result<Self, Self::Error> {
        Self::filtered(
            fields.query,
            fields.band,
            fields.locus_group,
            fields.limit,
            fields.offset,
        )
    }
}
impl GeneSearch {
    /// Check the required search selector and page bounds.
    pub fn new(
        query: Option<Text>,
        band: Option<Cytoband>,
        limit: u32,
        offset: u32,
    ) -> Result<Self, QueryInvalid> {
        Self::filtered(query, band, None, limit, offset)
    }
    /// Check the selector, optional locus-group filter and page bounds.
    pub fn filtered(
        query: Option<Text>,
        band: Option<Cytoband>,
        locus_group: Option<LocusGroup>,
        limit: u32,
        offset: u32,
    ) -> Result<Self, QueryInvalid> {
        if query.is_none() && band.is_none() {
            return Err(QueryInvalid::new(&["query", "band"]));
        }
        if !(MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        Ok(Self(SearchFields {
            query,
            band,
            locus_group,
            limit,
            offset,
        }))
    }
    /// Optional search text.
    pub fn query(&self) -> Option<&Text> {
        self.0.query.as_ref()
    }
    /// Optional band prefix.
    pub fn band(&self) -> Option<&Cytoband> {
        self.0.band.as_ref()
    }
    /// Optional HGNC locus group.
    pub fn locus_group(&self) -> Option<LocusGroup> {
        self.0.locus_group
    }
    /// Page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Page start.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
}
impl crate::Paged for GeneSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

pub(crate) fn present<'de, T: Deserialize<'de>, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

biodata::canonical_enum! {
    /// A section request word, including expansion to all landed sections.
    GeneSectionWord, SourceUnknown {
        /// Reviewed human proteins.
        Protein => "protein",
        /// CIViC gene curation.
        Civic => "civic",
        /// ClinGen validity and dosage.
        ClinGen => "clingen",
        /// GenCC submitter assessments.
        Gencc => "gencc",
        /// Population constraint metrics.
        Constraint => "constraint",
        /// Human Protein Atlas expression.
        Expression => "expression",
        /// Human Reactome pathways.
        Pathways => "pathways",
        /// Keyed OncoKB annotations.
        Oncokb => "oncokb",
        /// Every landed section.
        All => "all",
    }
}
biodata::canonical_enum! {
    /// A landed section that can be requested for a gene.
    GeneSection, SourceUnknown {
        /// Reviewed human proteins.
        Protein => "protein",
        /// CIViC gene curation.
        Civic => "civic",
        /// ClinGen validity and dosage.
        ClinGen => "clingen",
        /// GenCC submitter assessments.
        Gencc => "gencc",
        /// Population constraint metrics.
        Constraint => "constraint",
        /// Human Protein Atlas expression.
        Expression => "expression",
        /// Human Reactome pathways.
        Pathways => "pathways",
        /// Keyed OncoKB annotations.
        Oncokb => "oncokb",
    }
}

impl GeneSection {
    /// Publisher owning this independent section.
    pub const fn source(self) -> crate::Source {
        use crate::Source;
        match self {
            Self::Protein => Source::UniProt,
            Self::Civic => Source::Civic,
            Self::ClinGen => Source::ClinGen,
            Self::Gencc => Source::Gencc,
            Self::Constraint => Source::Gnomad,
            Self::Expression => Source::Hpa,
            Self::Pathways => Source::Reactome,
            Self::Oncokb => Source::Oncokb,
        }
    }
}
