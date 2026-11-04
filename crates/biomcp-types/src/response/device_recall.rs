use biodata::{DeviceRecallNumber, PartialDate, Text};
use serde::{Deserialize, Serialize};
/// Native medical device product recall header, independent of event grouping.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DeviceRecallHeader {
    /// Exact native product_res_number.
    pub recall_number: DeviceRecallNumber,
    /// Native res_event_number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_number: Option<Text>,
    /// Native recalling_firm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firm: Option<Text>,
    /// Native device category code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_code: Option<Text>,
    /// Native recall_status, without interpreting completion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Text>,
    /// Native event_date_initiated, preserving the source day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiated: Option<PartialDate>,
}
/// Exact recall details. Submission numbers belong only to this native recall record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(inline))]
pub struct DeviceRecallCard {
    /// Native recall identity and dates.
    pub header: DeviceRecallHeader,
    /// Native reason_for_recall.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Text>,
    /// Native root_cause_description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<Text>,
    /// Native product_description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_description: Option<Text>,
    /// Native distribution_pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution_pattern: Option<Text>,
    /// Classification from an exact enforcement recall_number match, absent without that match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<Text>,
    /// Top-level native 510(k) submission entries; harmonized openfda fields never enter this list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub k_numbers: Option<Box<[Text]>>,
    /// Top-level native PMA submission entries; no code-level identifiers enter this list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pma_numbers: Option<Box<[Text]>>,
}
