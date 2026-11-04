use crate::{DiagnosticGet, DiagnosticSearch, DiagnosticSection, Problem, ResponseBody};
use biodata::{
    CompanionDiagnosticRelation, GtrConditionRelation, GtrGeneRelation, GtrMethod, GtrTestId,
    PmaNumber, PmaSupplementCell, Text, WhoIvdProduct,
};
use serde::{Deserialize, Serialize};

/// Compact native laboratory context without authorization interpretation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiagnosticLaboratory {
    /// Native laboratory identifier.
    pub id: Text,
    /// Source laboratory name, absent when the native cell is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Native institution name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub institution: Option<Text>,
    /// Native country.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<Text>,
}
/// Small resolved GTR candidate; broad gene and condition arrays stay off the header.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GtrHeader {
    /// Complete source accession and version.
    pub id: GtrTestId,
    /// Native test name, absent when the cell is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Native laboratory context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laboratory: Option<DiagnosticLaboratory>,
    /// Source record URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Text,
}
/// A conflict is visible once without a selected name or card.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "identity_status", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DiagnosticCandidate {
    /// Native PMA submission; supplements retain their own identity.
    PmaSubmission {
        /// Exact native submission header.
        header: DiagnosticPmaHeader,
    },
    /// Native 510(k) record.
    Clearance {
        /// Native clearance fields.
        header: DiagnosticClearance,
    },
    /// One native product recall record; repeated event numbers remain separate records.
    DeviceRecall {
        /// Source-owned compact recall fields.
        header: super::DeviceRecallHeader,
    },
    /// One matched WHO product with cells from its first matching native row.
    WhoIvd {
        /// Exact source product identifier.
        id: biodata::WhoProductId,
        /// Unmodified product name cell.
        product_name: Box<str>,
        /// Unmodified manufacturer cell.
        manufacturer: Box<str>,
        /// Unmodified marker cell.
        marker: Box<str>,
    },
    /// One resolved native identity.
    Resolved {
        /// Compact source header.
        header: GtrHeader,
        /// Complete native relation that matched the source selector.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        matched: Option<biodata::GtrTestRelation>,
    },
    /// A native relation remains visible without a current card.
    MissingCard {
        /// Exact relation-owned test accession and version.
        id: GtrTestId,
        /// The original matched relation; no manufactured header.
        matched: biodata::GtrTestRelation,
    },
    /// Several incompatible native rows with the same exact accession/version.
    Conflict {
        /// Exact conflicting identity.
        id: GtrTestId,
        /// Explicit source identity problem; no chosen winner.
        problem: Problem,
    },
}
/// Native flags retain distinct source meanings and exact source codes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GtrNativeStatus {
    /// Current version flag, not current clinical availability.
    pub now_current: Box<str>,
    /// Native test current status.
    #[serde(rename = "test_currStat")]
    pub test_current: Box<str>,
    /// Native test public status.
    #[serde(rename = "test_pubStat")]
    pub test_public: Box<str>,
    /// Native laboratory current status.
    #[serde(rename = "lab_currStat")]
    pub laboratory_current: Box<str>,
    /// Native laboratory public status.
    #[serde(rename = "lab_pubStat")]
    pub laboratory_public: Box<str>,
}
/// Exact GTR card retains native context but never embeds its broad relation lists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(inline))]
pub struct GtrCard {
    /// Source identity header.
    pub header: GtrHeader,
    /// Separate native flags.
    pub native_status: GtrNativeStatus,
    /// Native revision date; not a local retrieval date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_touch_date: Option<Text>,
    /// Source creation date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_create_date: Option<Text>,
    /// Native deletion cell, including empty.
    pub test_deletion_date: Box<str>,
    /// Native manufacturer test name, including empty.
    pub manufacturer_test_name: Box<str>,
    /// Native development description, without regulatory inference.
    pub test_development: Box<str>,
    /// Native laboratory test identifier, including empty.
    pub lab_test_id: Box<str>,
    /// Native order code, including empty.
    pub test_order_code: Box<str>,
    /// Native CLIA number; never authorization.
    pub clia_number: Box<str>,
    /// Whole original license list, not paired with numbers by position.
    pub state_licenses: Box<str>,
    /// Whole original license number list, independently retained.
    pub state_license_numbers: Box<str>,
    /// Source-supported named pages.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub sections: Box<[DiagnosticSection]>,
}
/// Exactly one source-owned diagnostic card; conflicts have no card variant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum DiagnosticCard {
    /// Exact native medical device recall with an optional exact enforcement classification.
    DeviceRecall(Box<super::DeviceRecallCard>),
    /// Exact resolved GTR accession/version.
    GtrTest(Box<GtrCard>),
    /// All native rows under one WHO Product ID.
    WhoIvdProduct(WhoIvdProduct),
    /// Exact native FDA base or supplement, never an application-wide inferred claim.
    PmaSubmission(DiagnosticPmaCard),
    /// Exact native 510(k) clearance, separate from PMA approval.
    Clearance(DiagnosticClearance),
}
impl ResponseBody for DiagnosticCard {
    const SCHEMA: &'static str = "biomcp.diagnostic-card/4";
    type Query = DiagnosticGet;
}
/// One bounded regulatory fragment from the exact selected native submission.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiagnosticRegulatoryFragment {
    /// Exact selected native record and its dated regulatory metadata.
    pub submission: DiagnosticPmaHeader,
    /// Native metadata for this exact record, without a duplicated whole statement.
    pub metadata: DiagnosticPmaRegulatory,
    /// Native text field being continued.
    pub field: DiagnosticRegulatoryField,
    /// Fragment offset, independent of characters and supplement rows.
    pub offset: u32,
    /// Total mechanical fragments.
    pub total: u32,
    /// Complete unmodified text fragment; omitted when the source statement is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub text: Option<Box<str>>,
}
biodata::wire_enum! {
    /// Regulatory text available in this source tier.
    DiagnosticRegulatoryField { #[doc = "Native ApprovalOrderStatement spelling."] ApprovalOrderStatement => "ao_statement" }
}
biodata::wire_enum! {
    /// Meaning of the envelope's total for a source-owned diagnostic page.
    DiagnosticCountUnit {
        /// Native product recall records, without deduplication by event or harm inference.
        RecallRecords => "recall-records",
        /// Native PMA submissions or 510(k) clearances.
        DeviceSubmissions => "device-submissions",
        /// Distinct WHO Product IDs, grouping repeated code rows.
        ProductIdentities => "product-identities",
        /// Distinct exact GTR accession and version identities.
        TestIdentities => "test-identities",
        /// Native relation rows, including repeated IDs and label-only rows.
        NativeRelationRows => "native-relation-rows",
        /// Independent source list entries, without positional pairing.
        NativeMethodEntries => "native-method-entries"
    }
}

page!(
    diagnostic,
    DiagnosticPage,
    DiagnosticCandidate,
    DiagnosticSearch,
    "biomcp.diagnostic-page/5",
    "Source-ordered native diagnostic and medical-device submission candidates."
);
page!(
    diagnostic,
    DiagnosticGenes,
    GtrGeneRelation,
    DiagnosticGet,
    "biomcp.diagnostic-genes/5",
    "Native GTR gene relation rows without variant-coverage inference."
);
page!(
    diagnostic,
    DiagnosticConditions,
    GtrConditionRelation,
    DiagnosticGet,
    "biomcp.diagnostic-conditions/5",
    "Native GTR condition rows without ontology or clinical conversion."
);
page!(
    diagnostic,
    DiagnosticMethods,
    GtrMethod,
    DiagnosticGet,
    "biomcp.diagnostic-methods/4",
    "Independent native method entries without positional pairing."
);
page!(
    diagnostic,
    DiagnosticRegulatory,
    DiagnosticRegulatoryFragment,
    DiagnosticGet,
    "biomcp.diagnostic-regulatory/3",
    "Exact selected submission regulatory text fragments."
);
page!(
    diagnostic,
    DiagnosticSupplements,
    DiagnosticPmaHeader,
    DiagnosticGet,
    "biomcp.diagnostic-supplements/3",
    "Native supplement records retaining exact identity and independent dated decisions."
);
page!(
    diagnostic,
    DiagnosticCompanion,
    CompanionDiagnosticRelation,
    DiagnosticSearch,
    "biomcp.diagnostic-companion/4",
    "Whole distinct physical companion-table rows with cell-owned identifiers."
);
/// Compact exact PMA row identity; its native empty base cell is explicit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiagnosticPmaHeader {
    /// Native application.
    pub pma_number: PmaNumber,
    /// Absent differs from explicit empty base cell.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplement_number: Option<PmaSupplementCell>,
    /// Native trade_name when supplied, including explicit empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trade_name: Option<Box<str>>,
    /// Native applicant when supplied, including explicit empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicant: Option<Box<str>>,
    /// Native decision_date when supplied, including explicit empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_date: Option<Box<str>>,
    /// Native decision_code when supplied, including explicit empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_code: Option<Box<str>>,
    /// Exact source record URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Text,
}
/// Selected PMA card; statement text stays behind its bounded regulatory page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(inline))]
pub struct DiagnosticPmaCard {
    /// Exact selected source header.
    pub header: DiagnosticPmaHeader,
    /// Source-supported pages for this exact base or supplement.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub sections: Box<[DiagnosticSection]>,
}
/// Dated metadata belonging only to the selected submission.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiagnosticPmaRegulatory {
    /// Native date_received when supplied, including empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_received: Option<Box<str>>,
    /// Native fed_reg_notice_date when supplied, including empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fed_reg_notice_date: Option<Box<str>>,
    /// Native advisory_committee when supplied, including empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory_committee: Option<Box<str>>,
    /// Native advisory_committee_description when supplied, including empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory_committee_description: Option<Box<str>>,
    /// Native product_code when supplied, including empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_code: Option<Box<str>>,
    /// Native supplement_type when supplied, including empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplement_type: Option<Box<str>>,
    /// Native supplement_reason when supplied, including empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplement_reason: Option<Box<str>>,
}

/// Native 510(k) fields without harmonized or inferred clinical content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(inline))]
pub struct DiagnosticClearance {
    /// Exact native K number.
    pub k_number: biodata::DiagnosticDeviceId,
    /// Native device_name; missing and empty remain distinct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<Box<str>>,
    /// Native applicant; missing and empty remain distinct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicant: Option<Box<str>>,
    /// Native decision_code; missing and empty remain distinct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_code: Option<Box<str>>,
    /// Native decision_description; missing and empty remain distinct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_description: Option<Box<str>>,
    /// Native decision_date; missing and empty remain distinct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_date: Option<Box<str>>,
    /// Native date_received; missing and empty remain distinct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_received: Option<Box<str>>,
    /// Native product_code; missing and empty remain distinct.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_code: Option<Box<str>>,
}

/// FDA-published SSED text for one PMA base application.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiagnosticSummary {
    /// Mechanical document text with explicit extraction state.
    pub document: super::PdfText,
}
impl ResponseBody for DiagnosticSummary {
    const SCHEMA: &'static str = "biomcp.diagnostic-summary/1";
    type Query = DiagnosticGet;
}
