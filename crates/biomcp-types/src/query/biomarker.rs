//! Checked selectors for NCI biomarker terminology.
use crate::Paged;
use crate::QueryInvalid;
use biodata::{NcitId, Text};
use serde::{Deserialize, Serialize};
biodata::wire_enum! {
    /// Biomarker concept publisher.
    BiomarkerSource { /// NCI Clinical Trials Search.
        Nci => "nci" }
}
fn nci() -> BiomarkerSource {
    BiomarkerSource::Nci
}
fn limit() -> u32 {
    5
}
#[doc = "Find NCI biomarker concepts by name or synonym. The public tier returns codes, name, type, trial count, synonyms, \
    parent_ids, ancestor_ids and eligibility_criterion. These concepts differ from diagnostic --biomarker filters and \
    literal trial --variant text. Use search trial --source nci --biomarker-code CODE for trials that list an exact concept. \
    Native criteria do not establish patient eligibility."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct BiomarkerSearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Explicit keyed NCI source.
    source: BiomarkerSource,
    /// Native partial name or synonym search.
    #[serde(
        default,
        deserialize_with = "super::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    name: Option<Text>,
    /// Maximum headers per local page.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Offset within the complete retained observation.
    #[serde(default)]
    offset: u32,
    /// Snapshot identity retained by continuation commands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pool: Option<Text>,
}
crate::query::validation::checked_query!(BiomarkerSearch, SearchFields);
impl TryFrom<SearchFields> for BiomarkerSearch {
    type Error = QueryInvalid;
    fn try_from(value: SearchFields) -> Result<Self, Self::Error> {
        if !(1..=50).contains(&value.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        Ok(Self(value))
    }
}
impl BiomarkerSearch {
    /// Native name or synonym selector.
    pub fn name(&self) -> Option<&Text> {
        self.0.name.as_ref()
    }
    /// Local page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Local snapshot position.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Retained observation identity.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
    /// Continue the same observation.
    pub fn at_page(&self, offset: u32, pool: Text) -> Self {
        let mut q = self.clone();
        q.0.offset = offset;
        q.0.pool = Some(pool);
        q
    }

    /// Snapshot selectors exclude page controls.
    pub fn snapshot_query(&self) -> Self {
        let mut q = self.restart();
        q.0.limit = limit();
        q
    }
}
impl crate::Paged for BiomarkerSearch {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) {
        self.0.offset = 0;
    }
    fn clear_page_context(&mut self) {
        self.0.pool = None;
    }
}

#[doc = "Search NCI by exact NCIt code. Several native rows return candidates without choosing one. The public tier returns \
    codes, name, type, trial count, synonyms, parent_ids, ancestor_ids and eligibility_criterion. No clinical interpretation \
    is performed."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BiomarkerGet {
    /// Exact NCIt code, such as C96022.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    pub id: NcitId,
    /// NCI is the biomarker concept source.
    #[serde(default = "nci")]
    pub source: BiomarkerSource,
}
impl BiomarkerGet {
    /// Construct an exact native code lookup.
    pub fn one(id: NcitId) -> Self {
        Self { id, source: nci() }
    }
}

crate::query::validation::checked_query!(BiomarkerGet);
