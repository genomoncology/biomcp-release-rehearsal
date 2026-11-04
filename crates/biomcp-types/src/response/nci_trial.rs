//! NCI source fields keep their native names and vocabulary.
use crate::{NctId, Text, TrialGet, TrialSearch};
use serde::{Deserialize, Serialize};
/// NCI trial card without registry status or phase coercion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciTrial {
    /// Publisher NCI identifier.
    pub nci_id: Text,
    /// Registry cross-reference supplied by NCI.
    pub nct_id: NctId,
    /// Native title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brief_title: Option<Text>,
    /// Native status, without a registry translation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_trial_status: Option<Text>,
    /// Native phase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<super::NciTrialPhase>,
    /// First site with an exact case-insensitive organization name, in source order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_site: Option<NciMatchedSite>,
    /// Number of sites with that name, without deduplication; present only for facility searches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matching_sites: Option<u32>,
    /// Entries with the exact searched biomarker code, preserving source order and native fields.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub matched_biomarkers: Box<[NciTrialBiomarker]>,
    /// Native short summary, only on exact retrieval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief_summary: Option<crate::NciTextPage>,
    /// Publisher-provided disease concepts in source order.
    pub diseases: crate::TrialRecords<NciDiseaseRef>,
}
/// One NCI disease concept; names never substitute for an NCIt identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciDisease {
    /// Source concept name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Publisher NCIt identity.
    pub nci_thesaurus_concept_id: biodata::NcitId,
    /// Publisher coding provenance, without local interpretation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inclusion_indicator: Option<Text>,
    /// Native lead-disease marker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_lead_disease: Option<bool>,
    /// Original concept categories.
    #[serde(rename = "type", default)]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub kind: Box<[Text]>,
    /// Original immediate ancestors.
    #[serde(default)]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub parents: Box<[Text]>,
    /// Original alternative labels.
    #[serde(default)]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub synonyms: Box<[Text]>,
}
/// NCI search page in native order with native count.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciTrialPage {
    /// Source rows, never merged with another provider.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[NciTrial]>,
    /// Exact source parameters, excluding credentials.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub request: Box<[crate::TrialParameter]>,
    /// Exact condition identity and source-provided mapping, absent for literal text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_resolution: Option<crate::TrialConditionResolution>,
    /// Publisher attribution.
    pub attribution: Box<str>,
}
impl super::ResponseBody for NciTrialPage {
    const SCHEMA: &'static str = "biomcp.nci-trial-page/10";
    type Query = TrialSearch;
}
/// Independent exact NCI answers in caller order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciTrialCards {
    /// One independent answer per requested identity.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[NciTrialItem]>,
    /// Publisher attribution.
    pub attribution: Box<str>,
}
impl super::ResponseBody for NciTrialCards {
    const SCHEMA: &'static str = "biomcp.nci-trial-cards/8";
    type Query = TrialGet;
}
/// An exact NCI card and its source state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciTrialItem {
    /// Exact caller identity.
    pub input: NctId,
    /// Independent retrieval state.
    pub status: crate::Status,
    /// Native source card, absent after failure or confirmed absence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<NciTrial>,
    /// Original reply retrieval time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
    /// Explicitly requested, source-native sections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<crate::NciTrialSections>,
    /// Proven sections omitted from this request.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub more: Box<[crate::TrialSection]>,
    /// Requested sections without proven public-tier support.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub unsupported_sections: Box<[crate::TrialSection]>,
}

/// Compact source disease reference; the diseases section retains every coding field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciDiseaseRef {
    /// Native source name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Source NCIt identity.
    pub nci_thesaurus_concept_id: biodata::NcitId,
}

/// Compact source site observation; its presence does not establish enrollment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciMatchedSite {
    /// Published organization name.
    pub org_name: Text,
    /// Native city, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_city: Option<Text>,
    /// Native state or province, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_state_or_province: Option<Text>,
    /// Native recruitment status, without translation or enrollment inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recruitment_status: Option<Text>,
}

/// One listed NCI biomarker entry; native values do not establish patient eligibility.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NciTrialBiomarker {
    /// Exact source NCIt concept identity.
    pub nci_thesaurus_concept_id: biodata::NcitId,
    /// Native criterion, without rewriting or a positivity inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligibility_criterion: Option<Box<str>>,
    /// Native inclusion indicator, without interpretation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inclusion_indicator: Option<Box<str>>,
}
