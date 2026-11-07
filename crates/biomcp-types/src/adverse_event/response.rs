use super::{
    AdverseEventGet, AdverseEventSearch, AdverseEventSection, AdverseEventSource, ReportUnit,
};
use crate::{ComponentLabel, ResponseBody};
use biodata::{FaersSeriousness, ReportId, Text};
use serde::{Deserialize, Serialize};
mod counts;
mod rows;
pub use counts::{AdverseEventReactionCount, AdverseEventReactionCounts};
pub use rows::{
    AdverseEventDevice, AdverseEventDrug, AdverseEventIdentityCounts, AdverseEventNarrativeItem,
    AdverseEventPatient, AdverseEventReaction, AdverseEventReporter,
};
/// Small source report header without a drug title, flattened lists or count buckets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventHeader {
    /// Exact published source identity.
    pub id: ReportId,
    /// Explicit report family.
    pub source: AdverseEventSource,
    /// Native version string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Box<str>>,
    /// Native serious string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serious: Option<Box<str>>,
    /// Native event_type string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<Box<str>>,
    /// Native first_received string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_received: Option<Box<str>>,
}
biodata::wire_enum! {
/// Source reaction selector matching semantics.
AdverseEventReactionMatch {
    /// Analyzed-field phrase match, without exact-term equality.
    PhraseMatch => "phrase-match",
}
}
/// Ordered report headers; totals count reports with no population denominator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventPage {
    /// Source reaction buckets when aggregation was requested; never report header rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction_counts: Option<AdverseEventReactionCounts>,
    /// Explicit analyzed reaction selector semantics when a reaction was supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction_match: Option<AdverseEventReactionMatch>,
    /// Exact literal provider expression; reaction selectors use phrase match.
    pub source_query: Box<str>,
    /// The publisher last_updated date, separate from retrieval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_updated: Option<crate::SourceUpdated>,
    /// Original retrieval time in Unix seconds, absent for undated legacy recordings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
    /// Native ordered report candidates.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[AdverseEventHeader]>,
    /// Explicit reports unit for envelope page totals.
    pub unit: ReportUnit,
    /// Applicable component terms.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[ComponentLabel]>,
}
impl ResponseBody for AdverseEventPage {
    const SCHEMA: &'static str = "biomcp.adverse-event-page/6";
    type Query = AdverseEventSearch;
}
/// FAERS card with native report-level facts and independent collection lengths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventFaersCard {
    /// Exact selected report identity.
    pub header: AdverseEventHeader,
    /// Opaque checked retained-report snapshot identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<Text>,
    /// Native latest_received string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_received: Option<Box<str>>,
    /// Native latest_received_format string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_received_format: Option<Box<str>>,
    /// Native transmitted string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted: Option<Box<str>>,
    /// Native transmitted_format string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmitted_format: Option<Box<str>>,
    /// Native first_received_format string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_received_format: Option<Box<str>>,
    /// Independent source report flags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seriousness: Option<FaersSeriousness>,
    /// Fixed native patient strings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patient: Option<AdverseEventPatient>,
    /// Optional reporter facts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporter: Option<AdverseEventReporter>,
    /// Actual drug-array length, never count bucket.
    pub drug_count: u32,
    /// Actual reaction-array length, never causal pairs.
    pub reaction_count: u32,
    /// Publisher overview, never a next step.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub source_url: Text,
    /// Exact successful source request URL, never guessed.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub report_url: Text,
    /// Named sections available in this report.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub available_sections: Box<[AdverseEventSection]>,
    /// Applicable component terms.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[ComponentLabel]>,
}
impl ResponseBody for AdverseEventFaersCard {
    const SCHEMA: &'static str = "biomcp.adverse-event-faers-card/5";
    type Query = AdverseEventGet;
}
/// MAUDE card; native device/patient counts remain separate from array lengths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventMaudeCard {
    /// Exact selected report identity.
    pub header: AdverseEventHeader,
    /// Opaque checked retained-report snapshot identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<Text>,
    /// Native report_number string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_number: Option<Box<str>>,
    /// Native event_date string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_date: Option<Box<str>>,
    /// Native received string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received: Option<Box<str>>,
    /// Native report_date string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_date: Option<Box<str>>,
    /// Native date_report string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_report: Option<Box<str>>,
    /// Native date_report_to_fda string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_report_to_fda: Option<Box<str>>,
    /// Native number_devices_in_event string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_devices_in_event: Option<Box<str>>,
    /// Native number_patients_in_event string; present empty values remain present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_patients_in_event: Option<Box<str>>,
    /// Actual device-array length.
    pub device_count: u32,
    /// Actual narrative-array length.
    pub narrative_count: u32,
    /// Publisher overview, never a next step.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub source_url: Text,
    /// Exact successful source request URL, never guessed.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub report_url: Text,
    /// Named sections available in this report.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub available_sections: Box<[AdverseEventSection]>,
    /// Applicable component terms.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[ComponentLabel]>,
}
impl ResponseBody for AdverseEventMaudeCard {
    const SCHEMA: &'static str = "biomcp.adverse-event-maude-card/4";
    type Query = AdverseEventGet;
}
/// One complete-row page tied to the retained report snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "AdverseEventRecords_{T}"))]
pub struct AdverseEventRecords<T> {
    /// Source ordered whole rows.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub records: Box<[T]>,
    /// Native section offset.
    pub offset: u32,
    /// Complete selected section length, not report or patient count.
    pub total: u64,
}
macro_rules! section {
    ($name:ident, $row:ty, $field:ident, $schema:literal) => {
        /// A bounded report section; all rows belong to the exact header and snapshot.
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name {
            /// Exact source report identity and version.
            pub header: AdverseEventHeader,
            /// Opaque retained source snapshot.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub snapshot: Option<Text>,
            /// One page of complete native rows.
            pub $field: AdverseEventRecords<$row>,
            /// Applicable component terms.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub components: Box<[ComponentLabel]>,
        }
        impl ResponseBody for $name {
            const SCHEMA: &'static str = $schema;
            type Query = AdverseEventGet;
        }
    };
}
section!(
    AdverseEventDrugs,
    AdverseEventDrug,
    drugs,
    "biomcp.adverse-event-drugs/4"
);
section!(
    AdverseEventReactions,
    AdverseEventReaction,
    reactions,
    "biomcp.adverse-event-reactions/6"
);
section!(
    AdverseEventDevices,
    AdverseEventDevice,
    devices,
    "biomcp.adverse-event-devices/4"
);
section!(
    AdverseEventNarrative,
    AdverseEventNarrativeItem,
    narrative,
    "biomcp.adverse-event-narrative/4"
);
/// One selected original identifier array, owned by an absolute drug index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventIdentifierRecords {
    /// Absolute native drug-array index within this report snapshot.
    pub drug_index: u32,
    /// Published identifier strings without rewriting.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub records: Box<[Box<str>]>,
    /// Original array offset.
    pub offset: u32,
    /// Complete selected array length.
    pub total: u64,
}
macro_rules! identifiers {
    ($($variant:ident, $field:ident, $wire:literal);+ $(;)?) => {
        /// Exactly one closed named identifier page in its original report snapshot.
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(tag = "section")]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub enum AdverseEventIdentifiers {
            $(
                /// A single original identifier array.
                #[serde(rename = $wire)]
                $variant {
                    /// Exact report identity and version.
                    header: AdverseEventHeader,
                    /// Opaque checked report snapshot.
                    #[serde(default, skip_serializing_if = "Option::is_none")]
                    snapshot: Option<Text>,
                    /// One complete-row page, owned by one native drug row.
                    $field: AdverseEventIdentifierRecords,
                    /// Applicable component terms.
                    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
                    components: Box<[ComponentLabel]>,
                },
            )+
        }
    };
}
identifiers! {
    Unii, unii, "unii";
    Rxcui, rxcui, "rxcui";
    ProductNdc, product_ndc, "product-ndc";
    PackageNdc, package_ndc, "package-ndc";
    SplId, spl_id, "spl-id";
    SplSetId, spl_set_id, "spl-set-id";
    ApplicationNumber, application_number, "application-number";
}
impl ResponseBody for AdverseEventIdentifiers {
    const SCHEMA: &'static str = "biomcp.adverse-event-identifiers/4";
    type Query = AdverseEventGet;
}
