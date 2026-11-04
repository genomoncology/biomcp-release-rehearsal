use super::{PathwaySource, QueryInvalid, default_limit, valid_limit};
use crate::GeneInput;
use biodata::Text;
use serde::{Deserialize, Serialize};
#[doc = " Search Reactome (default) or WikiPathways human pathways. Membership names a source and \
    release; it implies no therapy, response or actionability. No enrichment is calculated."]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SearchFields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "SearchFields"))]
pub struct PathwaySearch(SearchFields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct SearchFields {
    /// Literal source query; exclusive with gene and top-level discovery.
    #[serde(
        default,
        deserialize_with = "super::super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text", extend("x-positional" = 0)))]
    query: Option<Text>,
    /// Human gene input for exact resolution and UniProt mapping; Reactome only.
    #[serde(
        default,
        deserialize_with = "super::super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "GeneInput"))]
    gene: Option<GeneInput>,
    /// Native human Reactome top-level discovery, exclusive with text and gene.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    top_level: bool,
    /// Sources in caller order; repeated sources take turns without title joins.
    #[serde(default = "default_sources", deserialize_with = "source_list")]
    #[cfg_attr(feature = "schema", schemars(length(min = 1, max = 2), extend("x-comma-separated" = true)))]
    source: Box<[PathwaySource]>,
    /// Five rows by default, one to fifty. Reactome rows equals this limit without refill.
    #[serde(default = "default_limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Native Reactome Start row or retained WikiPathways catalog offset.
    #[serde(default)]
    offset: u32,
    /// Retained snapshot identity for deterministic continuation.
    #[serde(
        default,
        deserialize_with = "super::super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pool: Option<Text>,
    /// Request a new retained observation.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    fresh: bool,
}
crate::query::validation::checked_query!(PathwaySearch, SearchFields);
impl TryFrom<SearchFields> for PathwaySearch {
    type Error = QueryInvalid;
    fn try_from(v: SearchFields) -> Result<Self, Self::Error> {
        let selectors = usize::from(v.query.is_some())
            + usize::from(v.gene.is_some())
            + usize::from(v.top_level);
        if selectors != 1 {
            return Err(QueryInvalid::new(&["query", "gene", "top_level"]));
        }
        if !valid_limit(v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        if v.source.is_empty() {
            return Err(QueryInvalid::new(&["source"]));
        }
        if v.source
            .iter()
            .enumerate()
            .any(|(i, s)| v.source.iter().take(i).any(|p| p == s))
        {
            return Err(QueryInvalid::new(&["source"]));
        }
        if v.source.contains(&PathwaySource::Wikipathways) && (v.gene.is_some() || v.top_level) {
            return Err(QueryInvalid::new(&["source", "gene", "top_level"]));
        }
        Ok(Self(v))
    }
}
impl PathwaySearch {
    /// Literal positional text.
    pub fn query(&self) -> Option<&Text> {
        self.0.query.as_ref()
    }
    /// Exact-resolution gene input.
    pub fn gene(&self) -> Option<&GeneInput> {
        self.0.gene.as_ref()
    }
    /// Whether native top-level discovery was selected.
    pub fn top_level(&self) -> bool {
        self.0.top_level
    }
    /// Explicit source, without fallback.
    pub fn source(&self) -> PathwaySource {
        self.0.source.first().copied().unwrap_or_default()
    }
    /// Native requested page size, which may exceed admitted rows.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Native result offset, independent of admitted row count.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Retained page snapshot.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
    /// Whether to refresh the observation.
    pub fn fresh(&self) -> bool {
        self.0.fresh
    }
    /// Bind continuation to an immutable retained observation.
    pub fn with_pool(&self, pool: Text) -> Self {
        let mut value = self.clone();
        value.0.pool = Some(pool);
        value.0.fresh = false;
        value
    }
}
impl crate::Paged for PathwaySearch {
    type Offset = u32;
    fn set_page_offset(&mut self, offset: Self::Offset) {
        self.0.offset = offset;
    }
    fn clear_page_context(&mut self) {
        self.0.pool = None;
        self.0.fresh = false;
    }
}

fn default_sources() -> Box<[PathwaySource]> {
    vec![PathwaySource::Reactome].into()
}
fn source_list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Box<[PathwaySource]>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Input {
        One(PathwaySource),
        Many(Box<[PathwaySource]>),
    }
    Ok(match Input::deserialize(d)? {
        Input::One(v) => vec![v].into(),
        Input::Many(v) => v,
    })
}
impl PathwaySearch {
    /// Requested source legs in their unchanged caller order.
    pub fn sources(&self) -> &[PathwaySource] {
        &self.0.source
    }
    /// Select Reactome without weakening the checked gene and top-level restrictions.
    pub fn reactome_leg(&self) -> Self {
        let mut value = self.clone();
        value.0.source = vec![PathwaySource::Reactome].into();
        value.0.pool = None;
        value
    }
}
