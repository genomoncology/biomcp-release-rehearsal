//! Posted ClinicalTrials.gov values, retaining source labels and numeric strings.
use serde::{Deserialize, Serialize};

/// Original trial card plus the posted results date from the same record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialPostedCard {
    /// Unchanged registry protocol card.
    #[serde(flatten)]
    pub protocol: biodata::TrialCard,
    /// First public posting date, distinct from submission and update dates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results_first_posted: Option<biodata::PartialDate>,
}

/// Whole native records in outcome, adverse-event metadata, group, serious and other order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "record", content = "data", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum TrialResultRecord {
    /// Posted measure, including group denominators, values and analyses.
    Outcome(TrialPostedOutcome),
    /// Native adverse-event collection scope.
    AdverseEvents(TrialAdverseEvents),
    /// Native adverse-event group and reported counts.
    AdverseEventGroup(TrialEventGroup),
    /// One serious adverse-event term with group statistics.
    SeriousEvent(TrialEvent),
    /// One other adverse-event term with group statistics.
    OtherEvent(TrialEvent),
}

/// Posted outcome measure.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialPostedOutcome {
    /// Native `type` field.
    #[serde(rename = "type")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<Box<str>>,
    /// Native `title` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Box<str>>,
    /// Native `description` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<str>>,
    /// Native `population_description` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub population_description: Option<Box<str>>,
    /// Native `reporting_status` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reporting_status: Option<Box<str>>,
    /// Native `param_type` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_type: Option<Box<str>>,
    /// Native `dispersion_type` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispersion_type: Option<Box<str>>,
    /// Native `unit_of_measure` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_of_measure: Option<Box<str>>,
    /// Native `time_frame` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_frame: Option<Box<str>>,
    /// Native `type_units` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_units: Option<Box<str>>,
    /// Native `denom_units` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denom_units: Option<Box<str>>,
    /// Native `calculate_pct` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calculate_pct: Option<bool>,
    /// Native `groups` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub groups: Box<[TrialResultGroup]>,
    /// Native `denoms` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub denoms: Box<[TrialResultDenominator]>,
    /// Native `classes` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub classes: Box<[TrialResultClass]>,
    /// Native `analyses` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub analyses: Box<[TrialResultAnalysis]>,
}

/// Native outcome group.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialResultGroup {
    /// Native `id` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Box<str>>,
    /// Native `title` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Box<str>>,
    /// Native `description` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<str>>,
}

/// Reported denominator with its original units.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialResultDenominator {
    /// Native `units` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units: Option<Box<str>>,
    /// Native `counts` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub counts: Box<[TrialResultCount]>,
}

/// Reported count, with source group identity.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialResultCount {
    /// Native `group_id` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<Box<str>>,
    /// Native `value` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Box<str>>,
}

/// Native outcome class; no regrouping.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialResultClass {
    /// Native `title` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Box<str>>,
    /// Native `denoms` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub denoms: Box<[TrialResultDenominator]>,
    /// Native `categories` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub categories: Box<[TrialResultCategory]>,
}

/// Native category containing reported measurements.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialResultCategory {
    /// Native `title` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Box<str>>,
    /// Native `measurements` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub measurements: Box<[TrialResultMeasurement]>,
}

/// Reported value and dispersion; strings are never recomputed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialResultMeasurement {
    /// Native `group_id` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<Box<str>>,
    /// Native `value` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Box<str>>,
    /// Native `spread` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spread: Option<Box<str>>,
    /// Native `lower_limit` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lower_limit: Option<Box<str>>,
    /// Native `upper_limit` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upper_limit: Option<Box<str>>,
    /// Native `comment` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<Box<str>>,
}

/// Source statistical analysis and comparison identities.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialResultAnalysis {
    /// Native `group_ids` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub group_ids: Box<[Box<str>]>,
    /// Native `group_description` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_description: Option<Box<str>>,
    /// Native `non_inferiority_type` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_inferiority_type: Option<Box<str>>,
    /// Native `non_inferiority_comment` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_inferiority_comment: Option<Box<str>>,
    /// Native `p_value` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_value: Option<Box<str>>,
    /// Native `p_value_comment` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_value_comment: Option<Box<str>>,
    /// Native `statistical_method` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistical_method: Option<Box<str>>,
    /// Native `statistical_comment` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistical_comment: Option<Box<str>>,
    /// Native `param_type` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_type: Option<Box<str>>,
    /// Native `param_value` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_value: Option<Box<str>>,
    /// Native `dispersion_type` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispersion_type: Option<Box<str>>,
    /// Native `dispersion_value` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispersion_value: Option<Box<str>>,
    /// Native `ci_pct_value` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_pct_value: Option<Box<str>>,
    /// Native `ci_num_sides` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_num_sides: Option<Box<str>>,
    /// Native `ci_lower_limit` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_lower_limit: Option<Box<str>>,
    /// Native `ci_upper_limit` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_upper_limit: Option<Box<str>>,
    /// Native `ci_lower_limit_comment` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_lower_limit_comment: Option<Box<str>>,
    /// Native `ci_upper_limit_comment` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_upper_limit_comment: Option<Box<str>>,
    /// Native `estimate_comment` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimate_comment: Option<Box<str>>,
    /// Native `other_analysis_description` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_analysis_description: Option<Box<str>>,
}

/// Native event reporting scope, repeated nowhere else.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialAdverseEvents {
    /// Native `frequency_threshold` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency_threshold: Option<Box<str>>,
    /// Native `time_frame` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_frame: Option<Box<str>>,
    /// Native `description` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<str>>,
}

/// Native event group and source counts; no rates are derived.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialEventGroup {
    /// Native `id` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Box<str>>,
    /// Native `title` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Box<str>>,
    /// Native `description` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<str>>,
    /// Native `deaths_num_affected` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deaths_num_affected: Option<u64>,
    /// Native `deaths_num_at_risk` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deaths_num_at_risk: Option<u64>,
    /// Native `serious_num_affected` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serious_num_affected: Option<u64>,
    /// Native `serious_num_at_risk` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serious_num_at_risk: Option<u64>,
    /// Native `other_num_affected` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_num_affected: Option<u64>,
    /// Native `other_num_at_risk` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_num_at_risk: Option<u64>,
}

/// Native event term and its group statistics.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialEvent {
    /// Native `term` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub term: Option<Box<str>>,
    /// Native `organ_system` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organ_system: Option<Box<str>>,
    /// Native `source_vocabulary` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_vocabulary: Option<Box<str>>,
    /// Native `assessment_type` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment_type: Option<Box<str>>,
    /// Native `notes` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<Box<str>>,
    /// Native `stats` field.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub stats: Box<[TrialEventStats]>,
}

/// Native adverse-event group statistics.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialEventStats {
    /// Native `group_id` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<Box<str>>,
    /// Native `num_events` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_events: Option<u64>,
    /// Native `num_affected` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_affected: Option<u64>,
    /// Native `num_at_risk` field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_at_risk: Option<u64>,
}
