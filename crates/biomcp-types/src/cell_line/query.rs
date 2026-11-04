use crate::QueryInvalid;
use biodata::{CellLineSourceId, CellosaurusAccession, ChemblCellId, Text};
use serde::{Deserialize, Serialize};
pub(super) fn default_limit() -> u32 {
    5
}
pub(super) fn check_search_limit(limit: u32) -> Result<(), QueryInvalid> {
    if !(1..=25).contains(&limit) {
        return Err(QueryInvalid::new(&["limit"]));
    }
    Ok(())
}
biodata::wire_enum! { described
    /// Named disclosure sections; all requests each bounded first page without an assay count.
    CellLineSection {
        /// Native display names.
        Names => "names",
        /// Independent species, diseases and qualified warnings.
        Identity => "identity",
        /// Complete source cross-reference rows.
        Xrefs => "xrefs",
        /// Complete native variation rows.
        Variants => "variants",
        /// Exact accession-linked ChEMBL cell metadata.
        Chembl => "chembl",
        /// Bounded first page of every named section.
        All => "all"
    }
}
/// An exact accession or a database-qualified source identity; never a name lookup.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "schema",
    schemars(
        with = "str",
        extend("pattern" = "^(CVCL_[A-Z0-9]{4}|[A-Za-z0-9_.-]+:[^\\s\\x00-\\x1f\\x7f-\\x9f]+)$")
    )
)]
pub enum CellLineInput {
    /// Primary accession; secondary accession resolution remains deferred.
    Accession(CellosaurusAccession),
    /// Exact source cross-reference requiring candidate and card verification.
    Source(CellLineSourceId),
}
/// One literal cell line search, with native order and no total claim.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct CellLineSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Literal source query from the positional argument.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    query: Text,
    /// Native page size.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 25)))]
    limit: u32,
    /// Native start offset.
    #[serde(default)]
    offset: u32,
}
crate::query::validation::checked_query!(CellLineSearch, SearchFields);
impl TryFrom<SearchFields> for CellLineSearch {
    type Error = QueryInvalid;
    fn try_from(v: SearchFields) -> Result<Self, Self::Error> {
        check_search_limit(v.limit)?;
        Ok(Self(v))
    }
}
impl CellLineSearch {
    pub(super) fn from_parts(query: Text, limit: u32, offset: u32) -> Self {
        Self(SearchFields {
            query,
            limit,
            offset,
        })
    }
    /// Literal checked search text.
    pub fn query(&self) -> &Text {
        &self.0.query
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
impl crate::Paged for CellLineSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
}

/// One exact identity and an optional bounded named section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct CellLineGet(GetFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// One exact primary accession or qualified source ID.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    id: CellLineInput,
    /// Omit for the default card; all returns bounded first pages without an assay count.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "CellLineSection", extend("x-positional" = 1)))]
    section: Option<CellLineSection>,
    /// Explicit row limit for each requested collection; native variation defaults to three.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "u32", range(min = 1, max = 25)))]
    section_limit: Option<u32>,
    /// Offset within each independently requested collection.
    #[serde(default)]
    section_offset: u32,
}
crate::query::validation::checked_query!(CellLineGet, GetFields);
impl TryFrom<GetFields> for CellLineGet {
    type Error = QueryInvalid;
    fn try_from(v: GetFields) -> Result<Self, Self::Error> {
        if v.section_limit.is_some_and(|n| !(1..=25).contains(&n)) {
            return Err(QueryInvalid::new(&["section_limit"]));
        }
        Ok(Self(v))
    }
}
impl CellLineGet {
    /// Exact primary accession with default disclosure.
    pub fn accession(id: CellosaurusAccession) -> Self {
        Self(GetFields {
            id: CellLineInput::Accession(id),
            section: None,
            section_limit: None,
            section_offset: 0,
        })
    }
    /// Select a named disclosure without changing identity.
    pub fn with_section(mut self, section: CellLineSection) -> Self {
        self.0.section = Some(section);
        self
    }
    /// Continue the same independently requested section.
    pub fn at_section_offset(&self, offset: u32) -> Self {
        let mut query = self.clone();
        query.0.section_offset = offset;
        query
    }
    /// Caller identity without normalization.
    pub fn id(&self) -> &CellLineInput {
        &self.0.id
    }
    /// Named disclosure, if requested.
    pub fn section(&self) -> Option<CellLineSection> {
        self.0.section
    }
    /// Explicit limit; absence lets each collection use its own default.
    pub fn section_limit(&self) -> Option<u32> {
        self.0.section_limit
    }
    /// Offset in each independent collection.
    pub fn section_offset(&self) -> u32 {
        self.0.section_offset
    }
}
/// A separate assay-count request, never implicit in metadata or all.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineAssayCountQuery {
    /// Exact ChEMBL cell identity returned by metadata.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    pub id: ChemblCellId,
}

crate::query::validation::checked_query!(CellLineAssayCountQuery);

impl CellLineSection {
    /// Publishers owning the requested data, separate from identity resolution.
    pub const fn sources(self) -> &'static [crate::Source] {
        use crate::Source;
        match self {
            Self::Chembl => &[Source::Chembl],
            Self::Names | Self::Identity | Self::Xrefs | Self::Variants => &[Source::Cellosaurus],
            Self::All => &[Source::Cellosaurus, Source::Chembl],
        }
    }
}
