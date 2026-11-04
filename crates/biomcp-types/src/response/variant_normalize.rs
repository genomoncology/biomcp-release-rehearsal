use crate::{Source, VariantNormalize, VariantResolution};
use serde::{Deserialize, Serialize};
/// Independent service results without local correction or transcript selection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantNormalization {
    /// Service results in declared order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub results: Box<[NormalizationResult]>,
}
impl super::ResponseBody for VariantNormalization {
    const SCHEMA: &'static str = "biomcp.variant-normalization/8";
    type Query = VariantNormalize;
}
/// One provider's own descriptions, messages and version metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NormalizationResult {
    /// Responsible service.
    pub source: Source,
    /// Provider outcome; warning rejection remains distinct from transport failure.
    pub outcome: VariantResolution,
    /// Source normalized description, without local edits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized: Option<biodata::SourceText>,
    /// Source corrected description, without local edits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected: Option<biodata::SourceText>,
    /// Mapping rows in native service order, labeled by their provider field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub mappings: Box<[NormalizationField]>,
    /// Count before mapping pagination.
    pub total_mappings: u32,
    /// Provider messages with original codes, text and category.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub messages: Box<[NormalizationMessage]>,
    /// Count before provider message pagination.
    #[serde(default)]
    pub total_messages: u32,
    /// Version fields exactly as returned; absent when the service states none.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub version: Box<[NormalizationField]>,
}
/// A provider field name and its unchanged text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NormalizationField {
    /// Native field path or assembly label.
    pub field: biodata::SourceText,
    /// Unchanged source text.
    pub value: biodata::SourceText,
}
/// Provider warning, error or informational message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NormalizationMessage {
    /// Native message category.
    pub category: biodata::SourceText,
    /// Native code when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<biodata::SourceText>,
    /// Unchanged message body, including whitespace.
    pub text: biodata::SourceText,
}
impl NormalizationResult {
    /// An unavailable service has no fabricated descriptions or version.
    pub fn unavailable(source: Source) -> Self {
        Self {
            source,
            outcome: VariantResolution::Unavailable,
            normalized: None,
            corrected: None,
            mappings: Box::default(),
            total_mappings: 0,
            messages: Box::default(),
            total_messages: 0,
            version: Box::default(),
        }
    }
}
