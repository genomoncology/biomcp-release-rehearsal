use super::{DrugGet, DrugSearch, DrugSection, DrugSource};
use crate::{ResponseBody, Section, Step};
use biodata::{
    AuthorizationEvent, DrugId, DrugIdentityLevel, DrugIdentityRelation, DrugIndicationClaim,
    DrugInteractionClaim, DrugLabelReference, DrugTargetRelation, PartialDate, Text,
};
use serde::{Deserialize, Serialize};
/// One source candidate, without authorization, targets, label text or interactions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugHeader {
    /// Exact name choice, ranking rule and bounded alternative commands; omitted for native ID calls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_resolution: Option<DrugNameResolution>,
    /// Published native identifier.
    pub id: DrugId,
    /// The single source responsible for this header.
    pub source: DrugSource,
    /// Original source display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pub name: Option<Text>,
    /// Level established only by the endpoint or record type.
    pub identity_level: DrugIdentityLevel,
    /// Complete native name fields matching the search input; omitted outside label searches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_name: Option<super::DrugNameMatch>,
    /// Native record type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Text>,
    /// Native source revision, omitted when the source supplies none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
    /// Source record URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Option<Text>,
}
/// Exact source identity and explicit native links; deeper facts require named sections.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugCard {
    /// Selected source-native identity.
    pub header: DrugHeader,
    /// Source-native alternate names, without local alias expansion.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub alternate_names: Box<[Text]>,
    /// The alternate names the source types as official: INN, USAN, BAN, JAN or FDA.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub official_names: Box<[Text]>,
    /// Explicit source cross-references with their native relation and provenance.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub cross_references: Box<[DrugIdentityRelation]>,
    /// Explicit native parent links.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub parents: Box<[DrugIdentityRelation]>,
    /// Explicit native product links.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub products: Box<[DrugIdentityRelation]>,
    /// Explicit native formulation links.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub formulations: Box<[DrugIdentityRelation]>,
    /// Source record version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
    /// Source-stated update date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<PartialDate>,
    /// Actual retrieval date, separate from source publication or update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved: Option<PartialDate>,
    /// Only disclosures available for this selected source identity.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub available_sections: Box<[DrugSection]>,
    /// Typed actions preserving exact source and identity.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub next: Box<[Step]>,
}
/// A single source candidate page in native order, without selecting the first hit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugPage {
    /// Native candidates, preserving their separate identity levels.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub items: Box<[DrugHeader]>,
    /// Native total when supplied; absent remains unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}
/// One bounded native row page, without fabricating an unknown total.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
#[cfg_attr(feature = "schema", schemars(rename = "DrugRecords_{T}"))]
pub struct DrugRecords<T> {
    /// Native source rows in order.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub records: Box<[T]>,
    /// Source row offset.
    pub offset: u32,
    /// Native total only when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// Native source truncation indicator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
    /// Dataset or bundle revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
    /// Actual retrieval date, never substituted for source revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved: Option<PartialDate>,
}
/// One slice of a native label array element, partitioning the page's unchanged text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugLabelFragment {
    /// Zero-based native array element; empty elements have a zero-character fragment.
    pub source_part: u32,
    /// Unicode scalar offset inside that original array element.
    pub source_offset: u32,
    /// Consecutive Unicode scalars consumed from the page text for this fragment.
    pub characters: u32,
}
/// A mechanical continuation of one native label heading.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugLabelText {
    /// Exact source label set and revision.
    pub label: DrugLabelReference,
    /// Exact native label heading.
    pub heading: Text,
    /// Original text slice, preserving whitespace and wording.
    pub text: Box<str>,
    /// Original source reply retrieval date; absent for legacy captures without a timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved: Option<PartialDate>,
    /// Native array boundaries; consume the flat text in this order without separators.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub fragments: Box<[DrugLabelFragment]>,
    /// Zero-based bounded text fragment offset; section_limit is a maximum fragment count.
    pub offset: u32,
    /// Total bounded text fragments, including fragments for empty native array elements.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}
/// One explicit native label heading and text continuation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugLabel {
    /// Selected native identity, checked independently of section facts.
    pub identity: DrugHeader,
    /// Existing typed section state, preserving empty, unavailable and skipped outcomes.
    pub label: Section<DrugLabelText>,
}
/// Native target and mechanism relations, without a clinical indication inference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugTargets {
    /// Selected native identity, checked independently of section facts.
    pub identity: DrugHeader,
    /// Existing typed section state, preserving empty, unavailable and skipped outcomes.
    pub targets: Section<DrugRecords<DrugTargetRelation>>,
}
/// Source-native indication wording, populations and combinations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugIndications {
    /// Selected native identity, checked independently of section facts.
    pub identity: DrugHeader,
    /// Existing typed section state, preserving empty, unavailable and skipped outcomes.
    pub indications: Section<DrugRecords<DrugIndicationClaim>>,
    /// Independent FDA application events found by the source's drug-name lookup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regulatory_context: Option<DrugRegulatoryContext>,
}
/// Native FDA events beside phase facts, without an indication-level assertion.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugRegulatoryContext {
    /// False: the indication claims and FDA events have not been linked.
    pub indications_linked: bool,
    /// Plain statement of the name-only relationship and independent source facts.
    pub notice: Box<str>,
    /// Native FDA application count for the drug-name lookup.
    pub total_applications: u64,
    /// Source-owned application identities, products and dated submission events.
    pub applications: Box<[DrugApprovals]>,
}
/// Complete CIViC therapy/profile context with independent native claim continuations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugCivic {
    /// Selected native identity, checked independently of section facts.
    pub identity: DrugHeader,
    /// Existing typed section state, preserving empty, unavailable and skipped outcomes.
    pub civic: Section<super::DrugCivicStreams>,
}
/// Explicit native application/product history without a global approval flag.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugApprovals {
    /// Selected native identity, checked independently of section facts.
    pub identity: DrugHeader,
    /// Native products, filtered only by an explicit product number; independent of submissions.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub products: Box<[super::DrugFdaProduct]>,
    /// Native FDA dataset last_updated date, independent of submission dates and retrieval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_updated: Option<crate::SourceUpdated>,
    /// Existing typed section state, preserving empty, unavailable and skipped outcomes.
    pub approvals: Section<DrugRecords<super::DrugFdaSubmission>>,
    /// Presentation order of application submissions, without interpreting their statuses.
    pub order: Box<str>,
}
impl DrugApprovals {
    /// Shared order descriptor for every approvals page.
    pub const ORDER: &'static str =
        "submission status date descending, then submission number descending; missing dates last";
}
/// Source-scoped regulator events preserving date roles and original status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugRegulatory {
    /// Selected native identity, checked independently of section facts.
    pub identity: DrugHeader,
    /// Existing typed section state, preserving empty, unavailable and skipped outcomes.
    pub regulatory: Section<DrugRecords<AuthorizationEvent>>,
}
/// Native DDInter partner rows and levels without invented management text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugInteractions {
    /// Selected native identity, checked independently of section facts.
    pub identity: DrugHeader,
    /// Existing typed section state, preserving empty, unavailable and skipped outcomes.
    pub interactions: Section<DrugRecords<DrugInteractionClaim>>,
}
impl ResponseBody for DrugPage {
    const SCHEMA: &'static str = "biomcp.drug-page/7";
    type Query = DrugSearch;
}
impl ResponseBody for DrugCard {
    const SCHEMA: &'static str = "biomcp.drug-card/6";
    type Query = DrugGet;
}
impl ResponseBody for DrugLabel {
    const SCHEMA: &'static str = "biomcp.drug-label/7";
    type Query = DrugGet;
}
impl ResponseBody for DrugTargets {
    const SCHEMA: &'static str = "biomcp.drug-targets/8";
    type Query = DrugGet;
}
impl ResponseBody for DrugIndications {
    const SCHEMA: &'static str = "biomcp.drug-indications/8";
    type Query = DrugGet;
}
impl ResponseBody for DrugCivic {
    const SCHEMA: &'static str = "biomcp.drug-civic/11";
    type Query = DrugGet;
}
impl ResponseBody for DrugApprovals {
    const SCHEMA: &'static str = "biomcp.drug-approvals/8";
    type Query = DrugGet;
}
impl ResponseBody for DrugRegulatory {
    const SCHEMA: &'static str = "biomcp.drug-regulatory/6";
    type Query = DrugGet;
}
impl ResponseBody for DrugInteractions {
    const SCHEMA: &'static str = "biomcp.drug-interactions/8";
    type Query = DrugGet;
}

/// A mechanical page of the selected document's complete checked native links.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugXrefs {
    /// Exact source-selected identity, without choosing any linked product.
    pub identity: DrugHeader,
    /// Native links with an exact link count, original version and retrieval date.
    pub xrefs: Section<DrugRecords<DrugIdentityRelation>>,
}
impl ResponseBody for DrugXrefs {
    const SCHEMA: &'static str = "biomcp.drug-xrefs/5";
    type Query = DrugGet;
}

/// Exact source name choice; brand before generic, original FDA application then earliest approval, otherwise source order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugNameResolution {
    /// Original complete name matched case-insensitively.
    pub input: Text,
    /// Name basis selected after demoting a brand identical to its own ingredient.
    pub basis: super::DrugNameMatch,
    /// Applied ambiguity ordering, emitted only from fixed nonempty rule text.
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pub ranking: Box<str>,
    /// Total exact source records, including the selected record.
    pub total: u64,
    /// Up to five other exact records as ready get commands.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub alternatives: Box<[Step]>,
}
