//! Research compatibility projections; the CLI retains the entity schemas.
use crate::Status;
use serde::{Deserialize, Serialize};

/// A citable exact source record, without inferred identity or relevance.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ResearchHit {
    /// Exact quoted get argument list, excluding the get verb.
    pub id: biodata::Text,
    /// Original source title, or its exact identifier when no title is supplied.
    pub title: crate::Text,
    /// Nonempty canonical source record URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: crate::Text,
}
/// Forward-compatible envelope metadata; entity fields use their CLI schemas.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ResearchEnvelope {
    /// Original CLI schema identity; its schema describes all entity fields.
    pub schema: biodata::Text,
    /// Original command outcome.
    pub status: Status,
}
/// Search disclosures preserve the original CLI response without inventing hits.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ResearchSearchMetadata<M> {
    /// Complete original CLI envelope.
    pub envelope: M,
    /// Native rows that lack an exact citable get target.
    pub omitted_rows: u32,
    /// Explains where unprojected rows remain available.
    pub note: biodata::Text,
}
/// Research search response.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ResearchSearch<M> {
    /// Citable records in unchanged CLI order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<ResearchHit>,
    /// Original response and projection disclosures.
    pub metadata: ResearchSearchMetadata<M>,
}
/// Research fetch response using the existing CLI Markdown view.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ResearchFetch<M> {
    /// Unchanged caller identifier.
    pub id: biodata::SourceText,
    /// Original source title, or exact identifier.
    pub title: crate::Text,
    /// Complete Markdown view of the bounded CLI record.
    pub text: biodata::Markdown,
    /// Nonempty canonical source URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: crate::Text,
    /// Complete original CLI envelope.
    pub metadata: M,
}

impl<M> ResearchSearch<M> {
    /// Canonical schema identity shared by discovery and the generated artifact registry.
    pub const SCHEMA: &'static str = "biomcp.research-search/3";
}
impl<M> ResearchFetch<M> {
    /// Canonical schema identity shared by discovery and the generated artifact registry.
    pub const SCHEMA: &'static str = "biomcp.research-fetch/3";
}
