use super::{PhaseFilter, TrialSource};
use crate::{NciPhaseFilter, TrialStatus};
use serde::{Deserialize, Serialize};

// Publisher OpenAPI components.parameters.current_trial_status, checked 2026-10-29.
// https://clinicaltrialsapi.cancer.gov/api/v2/docs
biodata::wire_enum! {
    /// Exact NCI current_trial_status values, without registry translation.
    NciStatusFilter {
        /// Publisher Active label.
        Active => "Active",
        /// Publisher Administratively Complete label.
        AdministrativelyComplete => "Administratively Complete",
        /// Publisher Approved label.
        Approved => "Approved",
        /// Publisher Closed to Accrual label.
        ClosedToAccrual => "Closed to Accrual",
        /// Publisher Closed to Accrual and Intervention label.
        ClosedToAccrualAndIntervention => "Closed to Accrual and Intervention",
        /// Publisher Complete label.
        Complete => "Complete",
        /// Publisher INACTIVE label.
        Inactive => "INACTIVE",
        /// Publisher Enrolling by Invitation label.
        EnrollingByInvitation => "Enrolling by Invitation",
        /// Publisher Temporarily Closed to Accrual label.
        TemporarilyClosedToAccrual => "Temporarily Closed to Accrual",
        /// Publisher Temporarily Closed to Accrual and Intervention label.
        TemporarilyClosedToAccrualAndIntervention => "Temporarily Closed to Accrual and Intervention",
        /// Publisher Withdrawn label.
        Withdrawn => "Withdrawn",
    }
}

/// Source-qualified status input with the source checked by TrialSearch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TrialStatusFilter {
    /// ClinicalTrials.gov vocabulary.
    Registry(TrialStatus),
    /// NCI vocabulary.
    Nci(NciStatusFilter),
}
impl TrialStatusFilter {
    pub(super) fn valid_for(&self, source: TrialSource) -> bool {
        matches!(
            (self, source),
            (Self::Registry(_), TrialSource::Ctgov) | (Self::Nci(_), TrialSource::Nci)
        )
    }
}

/// Source-qualified phase input with the source checked by TrialSearch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TrialPhaseFilter {
    /// ClinicalTrials.gov vocabulary.
    Registry(PhaseFilter),
    /// NCI vocabulary.
    Nci(NciPhaseFilter),
}
impl TrialPhaseFilter {
    pub(super) fn valid_for(&self, source: TrialSource) -> bool {
        matches!(
            (self, source),
            (Self::Registry(_), TrialSource::Ctgov) | (Self::Nci(_), TrialSource::Nci)
        )
    }
}

#[cfg(feature = "schema")]
fn filter_schema<A: schemars::JsonSchema, B: schemars::JsonSchema>() -> schemars::Schema {
    let mut values = Vec::new();
    for schema in [schemars::schema_for!(A), schemars::schema_for!(B)] {
        if let Some(choices) = schema.get("enum").and_then(|value| value.as_array()) {
            values.extend(choices.iter().cloned());
        }
    }
    schemars::json_schema!({"type": "string", "enum": values})
}
#[cfg(feature = "schema")]
impl schemars::JsonSchema for TrialStatusFilter {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "TrialStatusFilter".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        filter_schema::<TrialStatus, NciStatusFilter>()
    }
}
#[cfg(feature = "schema")]
impl schemars::JsonSchema for TrialPhaseFilter {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "TrialPhaseFilter".into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        filter_schema::<PhaseFilter, NciPhaseFilter>()
    }
}
