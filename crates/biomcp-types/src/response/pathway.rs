use crate::{
    PathwayEventsQuery, PathwayGenesQuery, PathwayGet, PathwaySearch, PathwaySection, PoolInfo,
    ResponseBody,
};
use biodata::{Pathway, PathwayEventRef, PathwayId, PathwayMember, PathwaySpecies, Text};
use serde::{Deserialize, Serialize};
/// Small selectable native pathway identity, without its potentially long description.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PathwayHeader {
    /// Checked source-owned identity.
    pub id: PathwayId,
    /// Whole native name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Explicit native human species.
    pub species: PathwaySpecies,
    /// Whole native Reactome stIdVersion, separate from stable identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
    /// WikiPathways revision, separate from identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<Text>,
}
biodata::wire_enum! {
    /// Meaning of the collection total, independent of admitted rows on a page.
    PathwayTotalKind {
        #[doc = "Native source-reported total before exact page admission."] Source => "source",
        #[doc = "Number of observed members in the retained relation."] Observed => "observed",
        #[doc = "Distinct namespaced references, preserving isoform identifiers."] DistinctReferences => "distinct-references",
        #[doc = "Native GeneProduct data nodes, including repeated references."] DataNodes => "data-nodes"
    }
}
/// Native page position and explicit count provenance; a short admitted page can continue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PathwayPageInfo {
    /// Native source or retained relation offset.
    pub offset: u32,
    /// Whole admitted rows on this page.
    pub returned: u32,
    /// Source-reported or observed relation total, as qualified by total_kind.
    pub total: u64,
    /// Whether the total comes from the source or this retained collection.
    pub total_kind: PathwayTotalKind,
    /// Next native position, independent of returned rows; absent only at the end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<u32>,
    /// Snapshot identity and lifetime when retained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pool: Option<PoolInfo>,
}
/// Source-ordered header page; class admission never changes the native total.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PathwayPage {
    /// Whole admitted pathway headers in native order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[PathwayHeader]>,
    /// Native position and total provenance.
    pub page: PathwayPageInfo,
}
impl ResponseBody for PathwayPage {
    const SCHEMA: &'static str = "biomcp.pathway-page/8";
    type Query = PathwaySearch;
}
/// One checked source pathway with supported disclosures and whole native text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PathwayCard {
    /// Source-native identity, species, versions and requested description.
    pub pathway: Pathway,
    /// Native Reactome database release, separate from pathway version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<Text>,
    /// Whether the source itself shortened its description, rather than the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_truncated: Option<bool>,
    /// Source-supported named sections; WikiPathways excludes events.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub sections: Box<[PathwaySection]>,
    /// Requested bounded membership page, including its explicit source state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genes: Option<crate::Section<PathwayGenes>>,
    /// Requested bounded event page, including unresolved integer references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<crate::Section<PathwayEvents>>,
}
impl ResponseBody for PathwayCard {
    const SCHEMA: &'static str = "biomcp.pathway-card/10";
    type Query = PathwayGet;
}

/// Independent source-ordered distinct membership references and their observed counts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PathwayGenes {
    /// Exact source pathway owning this relation.
    pub header: PathwayHeader,
    /// Distinct namespaced source references in first occurrence order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[PathwayMember]>,
    /// Distinct-reference total and continuation.
    pub page: PathwayPageInfo,
    /// All reference occurrences across physical entities before distinct-reference grouping.
    pub reference_occurrences: u64,
    /// All native physical entities, never described as gene counts.
    pub physical_entities: u64,
}
impl ResponseBody for PathwayGenes {
    const SCHEMA: &'static str = "biomcp.pathway-genes/9";
    type Query = PathwayGenesQuery;
}
page!(
    pathway,
    PathwayEvents,
    PathwayEventRef,
    PathwayEventsQuery,
    "biomcp.pathway-events/6",
    "Independent source-ordered page of whole native references."
);
/// Receipt for one independently validated WikiPathways metadata catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WikiPathwaysCatalogReceipt {
    /// Hash of this catalog's retained native bytes.
    pub sha256: Text,
    /// Bytes in this catalog, not the other catalog or a GPML archive.
    pub bytes: u64,
    /// This catalog's retained revision.
    pub revision: Text,
    /// Source-stated human rows in this catalog.
    pub human_rows: u64,
    /// All native rows in this catalog before human filtering.
    pub total_rows: u64,
}
/// Two independently validated metadata catalogs; no catalog body in stdout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct WikiPathwaysSync {
    /// Source body bytes received in this synchronization; excludes replay.
    pub downloaded_bytes: u64,
    /// Search metadata catalog receipt.
    pub search: WikiPathwaysCatalogReceipt,
    /// Exact-get metadata catalog receipt.
    pub get: WikiPathwaysCatalogReceipt,
}

impl ResponseBody for WikiPathwaysSync {
    const SCHEMA: &'static str = "biomcp.wikipathways-sync/4";
    type Query = crate::SyncQuery;
}
