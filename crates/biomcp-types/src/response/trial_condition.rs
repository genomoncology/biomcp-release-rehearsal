use crate::{ComponentLabel, DiseaseConcept, Source};
use biodata::{Attribution, Curie, DiseaseRelation};
use serde::{Deserialize, Serialize};
/// Exact condition identity used for one trial request, without eligibility inference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TrialConditionResolution {
    /// Caller-selected CURIE, never a source-name search.
    pub input: Curie,
    /// Named exact resolver; absent for explicit NCIt input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolver: Option<Source>,
    /// Returned concept with its unchanged native primary label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concept: Option<DiseaseConcept>,
    /// Native relation proving a requested alias, when required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<DiseaseRelation>,
    /// Native NCIt mapping rows; duplicate targets keep their independent provenance.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub ncit_candidates: Box<[DiseaseRelation]>,
    /// Explicit or unique source-provided NCIt identifier actually sent to NCI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_ncit: Option<Curie>,
    /// Resolver record provenance, independent of the trial provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attribution: Option<Attribution>,
    /// Admitted resolver components and their own terms.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[ComponentLabel]>,
}
