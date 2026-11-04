//! Registered arms and interventions retain their original relationships.
use super::{Arms, Problems, TrialArm, TrialIntervention};

impl Arms {
    pub(super) fn rows(
        self,
        problems: &mut Problems,
        refusals: &mut crate::Refusal,
    ) -> (Box<[TrialArm]>, Box<[TrialIntervention]>) {
        (
            super::super::refusals::records(problems, refusals, "armGroups", self.arm_groups)
                .into_vec()
                .into_iter()
                .map(|v| TrialArm {
                    label: problems.required_native_text("armGroups.label", v.label),
                    kind: problems.field("armGroups.type", v.kind),
                    description: problems.field("armsInterventions.description", v.description),
                    intervention_names: problems
                        .native_texts("armGroups.interventionNames", v.intervention_names),
                })
                .collect(),
            super::super::refusals::records(
                problems,
                refusals,
                "interventions",
                self.interventions,
            )
            .into_vec()
            .into_iter()
            .map(|v| TrialIntervention {
                name: problems.required_native_text("interventions.name", v.name),
                kind: problems.field("interventions.type", v.kind),
                description: problems.field("armsInterventions.description", v.description),
                arm_group_labels: problems
                    .native_texts("interventions.armGroupLabels", v.arm_group_labels),
                other_names: problems.native_texts("interventions.otherNames", v.other_names),
            })
            .collect(),
        )
    }
}
