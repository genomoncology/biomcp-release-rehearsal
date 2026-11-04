//! NCI biomarker terminology retains native lists without interpretation.
use biodata::{NcitId, Text};
use serde::{Deserialize, Serialize};
/// One native biomarker header. Codes identify concepts, not unique result rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BiomarkerHeader {
    /// Published NCIt codes, in source order.
    pub codes: Box<[NcitId]>,
    /// Native biomarker concept name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Native type values; an empty source list is omitted.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub r#type: Box<[Text]>,
    /// Native NCI trial count across trial statuses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}
/// Native terminology card; source criteria remain uninterpreted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BiomarkerCard {
    /// Native concept identity and count.
    pub header: BiomarkerHeader,
    /// Native synonym values.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub synonyms: Option<Box<[Text]>>,
    /// Native direct parent codes.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub parent_ids: Option<Box<[NcitId]>>,
    /// Native ancestor codes.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub ancestor_ids: Option<Box<[NcitId]>>,
    /// Native eligibility criterion values, exactly as published.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub eligibility_criterion: Option<Box<[Text]>>,
}
/// Local page of a complete bounded source observation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BiomarkerPage {
    /// Native biomarker headers in source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[BiomarkerHeader]>,
}
impl crate::ResponseBody for BiomarkerPage {
    const SCHEMA: &'static str = "biomcp.biomarker-page/4";
    type Query = crate::BiomarkerSearch;
}
/// Exact code result; multiple rows are candidates, never a selected card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BiomarkerResult {
    /// The sole native row when exactly one row matched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<BiomarkerCard>,
    /// Every native candidate when several rows matched the code.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub candidates: Box<[BiomarkerCard]>,
}
impl crate::ResponseBody for BiomarkerResult {
    const SCHEMA: &'static str = "biomcp.biomarker-result/4";
    type Query = crate::BiomarkerGet;
}
