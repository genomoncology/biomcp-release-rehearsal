//! Preserve exact-code biomarker entries and disclose independent-filter mismatches.
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::{NcitId, Text};
use biomcp_types::{NciBiomarkerUse, NciTrial, NciTrialBiomarker, Source};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Biomarker {
    nci_thesaurus_concept_id: Option<Field<NcitId>>,
    eligibility_criterion: Option<Field<Box<str>>>,
    inclusion_indicator: Option<Field<Box<str>>>,
}
impl super::Trial {
    /// Keep native trial positions even when the requested use does not bind this code.
    pub fn biomarker_card(
        self,
        code: &NcitId,
        criterion: Option<NciBiomarkerUse>,
        mut card: NciTrial,
    ) -> Mapped<NciTrial> {
        let mut p = Problems {
            source: Source::Nci,
            record: Some(self.nci_id),
            list: Vec::new(),
        };
        let mut refusals = self.refusals;
        let mut entries = Vec::new();
        for row in p.records("biomarkers", self.biomarkers, &mut refusals) {
            let Some(id) = p.required_field(
                "biomarkers.nci_thesaurus_concept_id",
                row.nci_thesaurus_concept_id,
            ) else {
                refusals = crate::Refusal::Refused;
                continue;
            };
            if &id != code {
                continue;
            }
            entries.push(NciTrialBiomarker {
                nci_thesaurus_concept_id: id,
                eligibility_criterion: p.required_checked_native_text(
                    "biomarkers.eligibility_criterion",
                    row.eligibility_criterion,
                ),
                inclusion_indicator: p.required_checked_native_text(
                    "biomarkers.inclusion_indicator",
                    row.inclusion_indicator,
                ),
            });
        }
        if entries.is_empty() {
            p.required::<Text>("biomarkers.nci_thesaurus_concept_id", None);
        }
        if let Some(criterion) = criterion {
            let expected = criterion.as_str().to_owned();
            if !entries.iter().any(|entry| {
                entry
                    .eligibility_criterion
                    .as_ref()
                    .is_some_and(|v| v.as_ref() == expected)
            }) {
                p.required::<Text>("biomarkers.eligibility_criterion", None);
            }
        }
        card.matched_biomarkers = entries.into();
        Mapped {
            refusals,
            value: card,
            problems: p.list,
        }
    }
}
