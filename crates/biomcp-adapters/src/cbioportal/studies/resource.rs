//! Native resource metadata keeps its study and access context strict.
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Resource {
    resource_id: Option<crate::raw::Field<biodata::StudyResourceId>>,
    display_name: Option<crate::raw::Field<Box<str>>>,
    description: Option<crate::raw::Field<Box<str>>>,
    resource_type: Option<crate::raw::Field<biodata::Text>>,
    priority: Option<crate::raw::Field<biodata::Text>>,
    open_by_default: bool,
    cancer_study_identifier: biodata::StudyId,
    sample_count: Option<crate::raw::Field<u64>>,
    patient_count: Option<crate::raw::Field<u64>>,
}
impl Resource {
    pub(super) fn mapped(
        self,
        problems: &mut crate::mapped::Problems,
        refusals: &mut crate::Refusal,
    ) -> Option<biodata::StudyResourceCount> {
        let Some(resource_id) =
            problems.required_field("resourceCounts.resourceId", self.resource_id)
        else {
            *refusals = crate::Refusal::Refused;
            return None;
        };
        Some(biodata::StudyResourceCount {
            resource_id,
            display_name: problems.required_field("resourceCounts.displayName", self.display_name),
            description: problems.required_field("resourceCounts.description", self.description),
            resource_type: problems
                .required_field("resourceCounts.resourceType", self.resource_type),
            priority: problems.required_field("resourceCounts.priority", self.priority),
            open_by_default: self.open_by_default,
            cancer_study_identifier: self.cancer_study_identifier,
            sample_count: problems.required_field("resourceCounts.sampleCount", self.sample_count),
            patient_count: problems
                .required_field("resourceCounts.patientCount", self.patient_count),
        })
    }
}
