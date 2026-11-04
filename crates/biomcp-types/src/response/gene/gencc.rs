use crate::Text;
use serde::{Deserialize, Serialize};
/// GenCC records in published order without merged classifications.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GenccGene {
    /// Individual submissions in source order.
    pub submissions: Box<[GenccSubmission]>,
}
/// One source submission; no clinical validity is inferred.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GenccSubmission {
    /// Source submission identifier.
    pub id: Text,
    /// Source submission version.
    pub version: Text,
    /// Source disease identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease_id: Option<Text>,
    /// Source disease title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease: Option<Text>,
    /// Source classification identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification_id: Option<Text>,
    /// Source classification title without inference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<Text>,
    /// Source inheritance identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_of_inheritance_id: Option<Text>,
    /// Source inheritance title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode_of_inheritance: Option<Text>,
    /// Source submitter identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitter_id: Option<Text>,
    /// Source submitter title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitter: Option<Text>,
    /// Source submitted_as_date, preserving its spelling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluated_date: Option<Text>,
    /// Source submitted_run_date, preserving its spelling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted_date: Option<Text>,
}
