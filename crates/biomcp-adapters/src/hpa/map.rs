use super::raw;
use crate::mapped::push_problem;
use crate::mapped::record as record_text;
use crate::{Mapped, raw::Field};
use biodata::{EnsemblGeneId, Text};

use biomcp_types::{CancerPtpm, GeneExpression, Problem, RnaAbundance, Source, TissueNtpm};
use std::collections::BTreeMap;
pub(super) fn expression(raw: raw::Gene, id: &EnsemblGeneId) -> Mapped<GeneExpression> {
    let mut map = Mapper {
        id,
        problems: Vec::new(),
    };
    let value = GeneExpression {
        rna_tissue_specificity: crate::mapped::field_code_text(
            &mut map.problems,
            Source::Hpa,
            record_text(id.base()),
            "rna_tissue_specificity",
            raw.tissue_specificity,
        ),
        rna_tissue_distribution: crate::mapped::field_code_text(
            &mut map.problems,
            Source::Hpa,
            record_text(id.base()),
            "rna_tissue_distribution",
            raw.tissue_distribution,
        ),
        rna_tissue_ntpm: map
            .numbers(raw.ntpm, "rna_tissue_ntpm")
            .into_iter()
            .map(|(tissue, value)| TissueNtpm {
                tissue,
                ntpm: biodata::WithUnit {
                    value,
                    unit: Some(biomcp_types::RnaUnit::Ntpm),
                },
            })
            .collect(),
        rna_cancer_specificity: crate::mapped::field_code_text(
            &mut map.problems,
            Source::Hpa,
            record_text(id.base()),
            "rna_cancer_specificity",
            raw.cancer_specificity,
        ),
        rna_cancer_ptpm: map
            .numbers(raw.ptpm, "rna_cancer_ptpm")
            .into_iter()
            .map(|(tissue, value)| CancerPtpm {
                tissue,
                ptpm: biodata::WithUnit {
                    value,
                    unit: Some(biomcp_types::RnaUnit::Ptpm),
                },
            })
            .collect(),
        subcellular_main: map.list(raw.main, "subcellular_main"),
        subcellular_additional: map.list(raw.additional, "subcellular_additional"),
    };
    Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: map.problems,
    }
}
struct Mapper<'a> {
    id: &'a EnsemblGeneId,
    problems: Vec<Problem>,
}
impl Mapper<'_> {
    fn list(&mut self, raw: Option<Field<Vec<Field<String>>>>, field: &str) -> Box<[Text]> {
        match raw {
            None => Box::default(),
            Some(Field::Value(values)) => values
                .into_iter()
                .filter_map(|value| {
                    crate::mapped::field_code_text(
                        &mut self.problems,
                        Source::Hpa,
                        record_text(self.id.base()),
                        field,
                        Some(value),
                    )
                })
                .collect(),
            Some(_) => {
                push_problem(
                    &mut self.problems,
                    Source::Hpa,
                    record_text(self.id.base()),
                    field,
                );
                Box::default()
            }
        }
    }
    fn numbers(
        &mut self,
        raw: Option<Field<BTreeMap<String, Field<String>>>>,
        field: &str,
    ) -> Vec<(Text, RnaAbundance)> {
        let Some(raw) = raw else {
            return Vec::new();
        };
        let Field::Value(values) = raw else {
            push_problem(
                &mut self.problems,
                Source::Hpa,
                record_text(self.id.base()),
                field,
            );
            return Vec::new();
        };
        values
            .into_iter()
            .filter_map(|(name, value)| {
                let tissue = crate::mapped::field_text(
                    &mut self.problems,
                    Source::Hpa,
                    record_text(self.id.base()),
                    field,
                    Some(Field::Value(name)),
                );
                if let Some(tissue) = tissue
                    && let Field::Value(value) = value
                    && let Ok(number) = value.parse::<f64>()
                    && number >= 0.0
                    && let Ok(number) = RnaAbundance::try_from(number)
                {
                    return Some((tissue, number));
                }
                push_problem(
                    &mut self.problems,
                    Source::Hpa,
                    record_text(self.id.base()),
                    field,
                );
                None
            })
            .collect()
    }
}
