use crate::QueryInvalid;
use biodata::{AuthorId, Text};
use serde::{Deserialize, Serialize};
const MAX_INPUTS: usize = 20;
fn default_limit() -> u32 {
    5
}
/// Author search supports one provider and its legacy spelling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "str", extend("enum" = ["semantic-scholar", "semanticscholar"])))]
pub enum AuthorSearchSource {
    /// Canonical Semantic Scholar source.
    #[default]
    #[serde(rename = "semantic-scholar", alias = "semanticscholar")]
    SemanticScholar,
}
biodata::wire_enum! { described
    /// Indexed-paper disclosure level.
    AuthorPaperSection {
        /// Native detailed metadata for the same paper window.
        Details => "details"
    }
}
/// Literal author name query and native window; positional text maps to query.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct AuthorSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Literal query from the positional argument.
    #[cfg_attr(feature = "schema", schemars(length(min = 1), extend("x-positional" = 0)))]
    query: Box<str>,
    /// Native author source.
    #[serde(default)]
    source: AuthorSearchSource,
    /// Bounded source page size.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 25)))]
    limit: u32,
    /// Native offset.
    #[serde(default)]
    offset: u32,
    /// Retained page identity.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pool: Option<Text>,
}
crate::query::validation::checked_query!(AuthorSearch, SearchFields);
impl TryFrom<SearchFields> for AuthorSearch {
    type Error = QueryInvalid;
    fn try_from(v: SearchFields) -> Result<Self, Self::Error> {
        if !(1..=25).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if v.query.trim().is_empty() {
            return Err(QueryInvalid::new(&["query"]));
        }
        Ok(Self(v))
    }
}
impl AuthorSearch {
    /// Literal source query.
    pub fn query(&self) -> &str {
        &self.0.query
    }

    /// Retain the checked snapshot identity for this native page.
    pub fn with_pool(&self, pool: Text) -> Self {
        let mut q = self.clone();
        q.0.pool = Some(pool);
        q
    }

    /// Selected native provider.
    pub fn source(&self) -> AuthorSearchSource {
        self.0.source
    }
    /// Bounded native window size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native start offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Retained window identity.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
}
impl crate::Paged for AuthorSearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
    fn clear_page_context(&mut self) {
        self.0.pool = None;
    }
}

/// Exact qualified records in caller order, with no name reconciliation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "GetFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "GetFields"))]
pub struct AuthorGet(GetFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct GetFields {
    /// One or more exact qualified IDs.
    #[cfg_attr(feature = "schema", schemars(length(min = 1, max = MAX_INPUTS), extend("x-positional" = 0)))]
    ids: Box<[AuthorId]>,
}
crate::query::validation::checked_query!(AuthorGet, GetFields);
impl TryFrom<GetFields> for AuthorGet {
    type Error = QueryInvalid;
    fn try_from(v: GetFields) -> Result<Self, Self::Error> {
        if !(1..=MAX_INPUTS).contains(&v.ids.len()) {
            return Err(QueryInvalid::new(&["ids"]));
        }
        Ok(Self(v))
    }
}
impl AuthorGet {
    /// Select one checked provider record.
    pub fn one(id: AuthorId) -> Self {
        Self(GetFields {
            ids: vec![id].into(),
        })
    }
    /// Qualified records in input order.
    pub fn ids(&self) -> &[AuthorId] {
        &self.0.ids
    }
}
/// One exact provider record's publication window. ORCID sends its whole public works list in one reply and pages groups locally.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PapersFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "PapersFields"))]
pub struct AuthorPapersQuery(PapersFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct PapersFields {
    /// Exact qualified provider record.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    id: AuthorId,
    /// Detailed disclosure when requested.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "AuthorPaperSection", extend("x-positional" = 1)))]
    section: Option<AuthorPaperSection>,
    /// Legacy alias for details; normalized on construction.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    full: bool,
    /// Bounded row count.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 25)))]
    limit: u32,
    /// Native or retained collection offset.
    #[serde(default)]
    offset: u32,
    /// Retained collection identity.
    #[serde(
        default,
        deserialize_with = "crate::query::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pool: Option<Text>,
}
crate::query::validation::checked_query!(AuthorPapersQuery, PapersFields);
impl TryFrom<PapersFields> for AuthorPapersQuery {
    type Error = QueryInvalid;
    fn try_from(mut v: PapersFields) -> Result<Self, Self::Error> {
        if !(1..=25).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if v.full {
            v.section = Some(AuthorPaperSection::Details);
            v.full = false;
        }
        Ok(Self(v))
    }
}
impl AuthorPapersQuery {
    /// Retain this collection identity.
    pub fn with_pool(&self, pool: Text) -> Self {
        let mut q = self.clone();
        q.0.pool = Some(pool);
        q
    }
    /// Start the default native paper window for a checked provider record.
    pub fn new(id: AuthorId) -> Self {
        Self(PapersFields {
            id,
            section: None,
            full: false,
            limit: default_limit(),
            offset: 0,
            pool: None,
        })
    }
    /// Exact qualified record.
    pub fn id(&self) -> &AuthorId {
        &self.0.id
    }
    /// Normalized disclosure selection.
    pub fn section(&self) -> Option<AuthorPaperSection> {
        self.0.section
    }
    /// Bounded row count.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Collection start offset.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Retained collection identity.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
}
impl crate::Paged for AuthorPapersQuery {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
    fn clear_page_context(&mut self) {
        self.0.pool = None;
    }
}
