use crate::{License, Text};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// Open-access status supplied by the paper-link provider.
    OpenAccessStatus, SourceUnknown {
        /// Publisher open access.
        Gold => "gold",
        /// Open article in a subscription journal.
        Hybrid => "hybrid",
        /// Free publisher copy without an open license claim.
        Bronze => "bronze",
        /// Repository copy.
        Green => "green",
        /// No open copy.
        Closed => "closed",
    }
}
/// Source-reported PDF link. Reading this section never downloads the file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PdfLink {
    /// Source publisher URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Text,
    /// Source access status.
    pub status: OpenAccessStatus,
    /// Normalized license; other grants no reuse rights.
    pub license: License,
}
/// Receipt for one file explicitly saved by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PdfSaved {
    /// Absolute destination path.
    pub path: Text,
    /// Number of saved bytes.
    pub bytes: u64,
    /// SHA-256 of the saved file.
    pub sha256: Text,
    /// Source-reported metadata.
    #[serde(flatten)]
    pub pdf: PdfLink,
}
/// Result of an explicit publisher PDF download.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticlePdfResult {
    /// Saved receipt or the reason no file was saved.
    pub file: crate::Section<PdfSaved>,
}
impl super::super::ResponseBody for ArticlePdfResult {
    const SCHEMA: &'static str = "biomcp.article-pdf/6";
    type Query = crate::ArticlePdf;
}
