//! Admit each native nonidentity field independently.
use crate::{native_text::NativeText, raw::Field};
use biodata::{NctId, Text};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Trial {
    pub nci_id: Text,
    pub nct_id: NctId,
    pub brief_title: Option<Field<NativeText>>,
    pub current_trial_status: Option<Field<Text>>,
    pub phase: Option<Field<biomcp_types::NciTrialPhase>>,
    pub brief_summary: Option<Field<Box<str>>>,
    pub diseases: Option<Field<Vec<Field<Disease>>>>,
    pub biomarkers: Option<Field<Vec<Field<super::trial_biomarker::Biomarker>>>>,
    pub sites: Option<Field<Vec<Field<super::sites::Site>>>>,
    pub eligibility: Option<Field<super::eligibility::Eligibility>>,
    pub arms: Option<Field<Vec<Field<Arm>>>>,
    pub outcome_measures: Option<Field<Vec<Field<Outcome>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Disease {
    pub name: Option<Field<NativeText>>,
    pub nci_thesaurus_concept_id: biodata::NcitId,
    pub inclusion_indicator: Option<Field<Text>>,
    pub is_lead_disease: Option<Field<bool>>,
    #[serde(rename = "type")]
    pub kind: Option<Field<Vec<Field<Text>>>>,
    pub parents: Option<Field<Vec<Field<Text>>>>,
    pub synonyms: Option<Field<Vec<Field<Text>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Arm {
    pub name: Option<Field<Box<str>>>,
    pub description: Option<Field<Box<str>>>,
    pub r#type: Option<Field<Text>>,
    pub interventions: Option<Field<Vec<Field<Intervention>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Intervention {
    pub name: Option<Field<NativeText>>,
    pub description: Option<Field<Box<str>>>,
    pub nci_thesaurus_concept_id: biodata::NcitId,
    pub r#type: Option<Field<Text>>,
    pub category: Option<Field<Text>>,
    pub inclusion_indicator: Option<Field<Text>>,
    pub parents: Option<Field<Vec<Field<Text>>>>,
    pub synonyms: Option<Field<Vec<Field<Text>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Outcome {
    pub name: Option<Field<Box<str>>>,
    pub description: Option<Field<Box<str>>>,
    pub timeframe: Option<Field<Box<str>>>,
    pub type_code: Option<Field<Box<str>>>,
}
