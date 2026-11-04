use crate::Text;
use serde::{Deserialize, Serialize};
/// CIViC's gene-level entry without evidence or variant lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivicGene {
    /// CIViC feature identifier.
    pub id: biodata::Identifier<u64>,
    /// Number of curated variants reported by the source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant_count: Option<u64>,
    /// Link to the source entry.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Text>,
    /// Complete source description, absent when the source has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Text>,
}
