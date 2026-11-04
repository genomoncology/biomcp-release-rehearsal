use crate::{Section, Text, TrialDocument};
use serde::{Deserialize, Serialize};
/// Receipt for exactly the original source bytes written by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialDocumentSaved {
    /// Absolute caller-owned path.
    pub path: Text,
    /// Saved byte count.
    pub bytes: u64,
    /// SHA-256 of the complete saved bytes.
    pub sha256: Text,
    /// Original manifest metadata and publisher URL.
    pub document: TrialDocument,
}
/// Original trial PDF download, never document text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialDocumentResult {
    /// Saved receipt, confirmed absence, or a source failure.
    pub file: Section<TrialDocumentSaved>,
}
impl super::ResponseBody for TrialDocumentResult {
    const SCHEMA: &'static str = "biomcp.trial-document/7";
    type Query = crate::TrialDocumentGet;
}
