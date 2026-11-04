//! Schema discovery entries shipped with an executable.
use serde::{Deserialize, Serialize};
/// Discoverable contracts shipped with this executable.
/// JSON reply readers ignore unknown fields and unknown growable enum values within a major.
/// Growable enums list known values in x-extensible-enum; query schemas remain closed.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SchemaCatalog {
    /// Every artifact in canonical identity order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schemas: Vec<SchemaEntry>,
}
impl super::ResponseBody for SchemaCatalog {
    const SCHEMA: &'static str = "biomcp.schema-catalog/4";
    type Query = super::EmptyQuery;
}
/// One schema identity with its own artifact documentation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SchemaEntry {
    /// Canonical versioned identity accepted by schema ID.
    pub id: String,
    /// One line of artifact documentation.
    pub description: String,
}
