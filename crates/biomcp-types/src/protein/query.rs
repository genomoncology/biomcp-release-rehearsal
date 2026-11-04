use crate::QueryInvalid;
use crate::query::present;
use biodata::{ComplexPortalId, GeneSymbol, Position1, Text, UniProtIsoform};
use serde::{Deserialize, Serialize};
fn limit() -> u32 {
    5
}
fn reviewed() -> bool {
    true
}
biodata::wire_enum! { described
    /// Named protein sections; all expands to bounded first pages.
    ProteinSection {
        /// Native UniProt features.
        Features => "features",
        /// Native InterPro entry matches.
        Domains => "domains",
        /// Native alternative products.
        Isoforms => "isoforms",
        /// Exact participant-admitted complex headers.
        Complexes => "complexes",
        /// Native structure cross-reference links.
        Structures => "structures",
        /// One whole native function text record with all source paragraphs.
        Function => "function",
        /// Direct exact-identity sequence and audit metadata.
        Sequence => "sequence",
        /// Bounded first page of every named section.
        All => "all"
    }
}
/// Exact source accession or a checked display gene symbol needing later resolution.
/// The schema describes the string surface; deserialization checks either identity grammar.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Text"))]
pub enum ProteinInput {
    /// Source accession, preserving an explicit isoform suffix.
    Accession(UniProtIsoform),
    /// A symbol requests candidates; no first-hit selection.
    Gene(GeneSymbol),
}
/// Search five reviewed human UniProt headers by literal text or exact gene.
/// No variant effect, structure, interaction or clinical meaning is inferred. No bioinformatics tool runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields", extend("anyOf" = [{"required":["query"]},{"required":["gene"]}])))]
pub struct ProteinSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Literal text, exclusive with gene.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text", extend("x-positional" = 0)))]
    query: Option<Text>,
    /// Exact native gene field, without identifier inference.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GeneSymbol"))]
    gene: Option<GeneSymbol>,
    /// Reviewed human scope stays enabled; unreviewed search is deferred.
    #[serde(default = "reviewed")]
    reviewed: bool,
    /// Five rows by default; no more than 25.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 25)))]
    limit: u32,
    /// Source/retained result offset.
    #[serde(default)]
    offset: u32,
    /// Exact retained collection identity.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pool: Option<Text>,
}
crate::query::validation::checked_query!(ProteinSearch, SearchFields);
impl TryFrom<SearchFields> for ProteinSearch {
    type Error = QueryInvalid;
    fn try_from(v: SearchFields) -> Result<Self, QueryInvalid> {
        if v.query.is_some() == v.gene.is_some() {
            return Err(QueryInvalid::new(&["query", "gene"]));
        }
        if !v.reviewed {
            return Err(QueryInvalid::new(&["reviewed"]));
        }
        if !(1..=25).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        Ok(Self(v))
    }
}
impl ProteinSearch {
    /// Walk one retained search without exposing a provider cursor.
    pub fn at_page(&self, pool: Text, offset: u32) -> Self {
        let mut q = self.clone();
        q.0.pool = Some(pool);
        q.0.offset = offset;
        q
    }

    /// Literal native text.
    pub fn query(&self) -> Option<&Text> {
        self.0.query.as_ref()
    }
    /// Exact native gene selector.
    pub fn gene(&self) -> Option<&GeneSymbol> {
        self.0.gene.as_ref()
    }
    /// Explicit reviewed scope.
    pub fn reviewed(&self) -> bool {
        self.0.reviewed
    }
    /// Bounded page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native result offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Retained pool when supplied.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
}
impl crate::Paged for ProteinSearch {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) {
        self.0.offset = 0;
    }
    fn clear_page_context(&mut self) {
        self.0.pool = None;
    }
}

/// Read source protein cards; a symbol returns candidates without choosing a protein.
/// Sections disclose native facts with independent row bounds; function keeps all paragraphs.
/// No variant effect, structure, interaction or clinical meaning is inferred. No bioinformatics tool runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct ProteinGet(GetFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// Source accessions or gene symbols, preserving caller order.
    #[cfg_attr(feature = "schema", schemars(length(min = 1, max = 20), extend("x-positional" = 0)))]
    input: Box<[ProteinInput]>,
    /// Select native sections or all bounded first pages; function keeps one whole text record.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    #[cfg_attr(feature = "schema", schemars(extend("x-trailing-section" = true)))]
    section: Box<[ProteinSection]>,
    /// First-page row limit; function keeps one whole native text record and does not page its paragraphs.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 25)))]
    section_limit: u32,
}
crate::query::validation::checked_query!(ProteinGet, GetFields);
impl TryFrom<GetFields> for ProteinGet {
    type Error = QueryInvalid;
    fn try_from(v: GetFields) -> Result<Self, QueryInvalid> {
        if !(1..=20).contains(&v.input.len()) {
            return Err(QueryInvalid::new(&["input"]));
        }
        if !(1..=25).contains(&v.section_limit) {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        Ok(Self(v))
    }
}
impl ProteinGet {
    /// Build a default card request without inferring an accession.
    pub fn new(input: Box<[ProteinInput]>) -> Result<Self, QueryInvalid> {
        GetFields {
            input,
            section: Box::default(),
            section_limit: limit(),
        }
        .try_into()
    }

    /// Select bounded native sections on the same exact input identities.
    pub fn with_section(&self, section: ProteinSection) -> Self {
        let mut q = self.clone();
        q.0.section = vec![section].into();
        q
    }

    /// Caller identities in their original order.
    pub fn input(&self) -> &[ProteinInput] {
        &self.0.input
    }
    /// Explicit requested sections.
    pub fn sections(&self) -> &[ProteinSection] {
        &self.0.section
    }
    /// Bound for each first section page.
    pub fn section_limit(&self) -> u32 {
        self.0.section_limit
    }
}
mod relation;
pub use relation::*;

impl ProteinSection {
    /// Publisher owning a named section; all expands across publishers.
    pub const fn source(self) -> Option<crate::Source> {
        use crate::Source;
        match self {
            Self::Domains => Some(Source::InterPro),
            Self::Complexes => Some(Source::ComplexPortal),
            Self::Features
            | Self::Isoforms
            | Self::Structures
            | Self::Function
            | Self::Sequence => Some(Source::UniProt),
            Self::All => None,
        }
    }
}
