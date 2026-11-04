use biodata::{DrugRecallNumber, PartialDate, Text};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// FDA's native recall classification, with no local ranking.
    DrugRecallClassification, SourceUnknown {
        /// Native Class I.
        ClassI => "Class I",
        /// Native Class II.
        ClassII => "Class II",
        /// Native Class III.
        ClassIII => "Class III",
    }
}
/// One native drug recall record, independent of its event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugRecallHeader {
    /// Native recall number.
    pub recall_number: DrugRecallNumber,
    /// Native FDA classification, without local interpretation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<Text>,
    /// Published recall status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Text>,
    /// Published recalling firm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firm: Option<Text>,
    /// Published product wording; harmonized names never enter this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_description: Option<Text>,
    /// Native recall initiation day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiated: Option<PartialDate>,
    /// Native enforcement report day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reported: Option<PartialDate>,
}
/// Native detail fields from the selected recall record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugRecallCard {
    /// Native recall header.
    pub header: DrugRecallHeader,
    /// Published reason for recall.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<Text>,
    /// Published distribution pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution_pattern: Option<Text>,
    /// Published product code information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_info: Option<Text>,
    /// Native voluntary or mandated status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voluntary_mandated: Option<Text>,
}
biodata::canonical_enum! {
    /// The native unit counted by drug enforcement searches.
    DrugRecallCountUnit, SourceUnknown {
        /// Product recall records, independent of events and patient counts.
        RecallRecords => "recall-records",
    }
}
/// A bounded page; totals count recall records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugRecallPage {
    /// Native count unit; no event deduplication or patient count.
    pub count_unit: DrugRecallCountUnit,
    /// Native source order, without event deduplication.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[DrugRecallHeader]>,
    /// Number of native recall records.
    pub total: u64,
}
impl crate::ResponseBody for DrugRecallPage {
    const SCHEMA: &'static str = "biomcp.drug-recall-page/4";
    type Query = super::DrugSearch;
}
impl crate::ResponseBody for DrugRecallCard {
    const SCHEMA: &'static str = "biomcp.drug-recall-card/3";
    type Query = super::DrugGet;
}
pub(super) fn validate_search(
    source: super::DrugSource,
    query: bool,
    filters: bool,
    from: Option<PartialDate>,
    to: Option<PartialDate>,
) -> Result<(), crate::QueryInvalid> {
    let recall = source == super::DrugSource::FdaDrugRecall;
    if !recall && (filters || !query) {
        return Err(crate::QueryInvalid::new(&[
            "source",
            "query",
            "firm",
            "classification",
            "status",
            "reason",
            "initiated_from",
            "initiated_to",
        ]));
    }
    for (name, date) in [("initiated_from", from), ("initiated_to", to)] {
        if date.is_some_and(|d| d.day().is_none()) {
            return Err(crate::QueryInvalid::new(&[name]));
        }
    }
    if from
        .zip(to)
        .is_some_and(|(a, b)| a.to_string() > b.to_string())
    {
        return Err(crate::QueryInvalid::new(&[
            "initiated_from",
            "initiated_to",
        ]));
    }
    Ok(())
}
