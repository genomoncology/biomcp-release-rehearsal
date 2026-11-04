use crate::{AccessionBase, FiniteNumber, Text};
use serde::{Deserialize, Serialize};
/// Source constraint metrics for the source's preferred transcript.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Constraint {
    /// Probability of loss-of-function intolerance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pli: Option<FiniteNumber>,
    /// Upper confidence bound of the observed-to-expected loss-of-function ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loeuf: Option<FiniteNumber>,
    /// Observed-to-expected loss-of-function ratio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oe_lof: Option<FiniteNumber>,
    /// Missense Z score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mis_z: Option<FiniteNumber>,
    /// Synonymous Z score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syn_z: Option<FiniteNumber>,
    /// Source flags, omitted when empty.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub flags: Box<[Text]>,
    /// Source MANE Select transcript, otherwise the source canonical transcript.
    pub transcript: AccessionBase,
    /// Source dataset family, not a minor release identifier.
    pub version: Text,
}
