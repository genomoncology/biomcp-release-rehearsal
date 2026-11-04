use crate::{NctId, PartialDate, Text, TrialPhase, TrialStatus};
use serde::{Deserialize, Serialize};
/// Small source-attributed trial discovery record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialHeader {
    /// Exact registry identity.
    pub nct_id: NctId,
    /// Source brief title, absent when missing or rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Text>,
    /// Registry overall status, absent when missing or rejected; independent of site status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TrialStatus>,
    /// Source phases; absent phases remain absent.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub phases: Box<[TrialPhase]>,
    /// Source condition names.
    pub conditions: crate::Capped<Text>,
    /// Source intervention names.
    pub interventions: crate::Capped<Text>,
    /// Last posted registry update, absent when missing or rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<PartialDate>,
    /// Publisher's canonical record URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Text,
}
