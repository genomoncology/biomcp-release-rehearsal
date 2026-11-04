//! Map nullable measurements without changing source allele or cohort identity.
use super::{CohortFrequency, Derivation, Group, Subset, Va, Variant, raw};
use crate::{Mapped, mapped::Problems, raw::Field};
use biomcp_types::Source;

pub(super) fn variant(value: Option<Field<raw::Variant>>, id: &str) -> Mapped<Option<Variant>> {
    let mut problems = Problems {
        source: Source::Gnomad,
        record: crate::mapped::record(id),
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let value = value.and_then(|value| {
        let Some(value) = problems.field("variant_id", Some(value)) else {
            refusals = crate::Refusal::Refused;
            return None;
        };
        if value.variant_id.as_str() != id {
            refusals = crate::Refusal::Refused;
            return problems.required("variant_id", None);
        }
        Some(Variant {
            variant_id: value.variant_id,
            exome: subset(value.exome, "exome", &mut problems),
            genome: subset(value.genome, "genome", &mut problems),
            joint: subset(value.joint, "joint", &mut problems),
            va: problems.field("va", value.va).map(|value| Va {
                exome: cohorts(value.exome, "va.exome", &mut problems),
                genome: cohorts(value.genome, "va.genome", &mut problems),
            }),
        })
    });
    Mapped {
        refusals,
        value,
        problems: problems.list,
    }
}

fn subset(
    value: Option<Field<raw::Subset>>,
    name: &str,
    problems: &mut Problems,
) -> Option<Subset> {
    let value = problems.field(name, value)?;
    let native_groups = problems.fields(&format!("{name}.populations.id"), value.populations);
    let mut identities = std::collections::BTreeMap::new();
    for group in &native_groups {
        *identities.entry(group.id.as_str()).or_insert(0) += 1;
    }
    let populations = native_groups
        .iter()
        .filter_map(|group| {
            if identities.get(group.id.as_str()) != Some(&1) {
                problems.required::<()>(&format!("{name}.populations.{}.id", group.id), None);
                return None;
            }
            Some(group.id.as_str().to_owned())
        })
        .collect::<std::collections::BTreeSet<_>>();
    let populations = native_groups
        .into_vec()
        .into_iter()
        .filter(|group| populations.contains(group.id.as_str()))
        .map(|group| {
            let ac = problems.field(&format!("{name}.populations.{}.ac", group.id), group.ac);
            let an = problems.field(&format!("{name}.populations.{}.an", group.id), group.an);
            if ac.zip(an).is_some_and(|(ac, an)| ac > an) {
                problems.required::<()>(&format!("{name}.populations.{}.counts", group.id), None);
            }
            Group {
                ac,
                an,
                id: group.id,
            }
        })
        .collect();
    let field = format!("{name}.filters");
    let filters = problems.field(&field, value.filters).map(|values| {
        values
            .into_iter()
            .filter_map(|value| problems.field(&field, Some(value)))
            .collect()
    });
    let ac = problems.field(&format!("{name}.ac"), value.ac);
    let an = problems.field(&format!("{name}.an"), value.an);
    if ac.zip(an).is_some_and(|(ac, an)| ac > an) {
        problems.required::<()>(&format!("{name}.counts"), None);
    }
    Some(Subset {
        ac,
        an,
        homozygote_count: problems
            .field(&format!("{name}.homozygote_count"), value.homozygote_count),
        filters,
        populations,
    })
}

fn cohorts(
    value: Option<Field<Vec<Field<raw::CohortFrequency>>>>,
    name: &str,
    problems: &mut Problems,
) -> Option<Vec<CohortFrequency>> {
    let value = problems.field(name, value)?;
    Some(
        value
            .into_iter()
            .filter_map(|value| {
                let value = problems.field(&format!("{name}.cohort.id"), Some(value))?;
                let derived_from = problems
                    .field(&format!("{name}.derivedFrom"), value.derived_from)
                    .map(|value| Derivation {
                        version: problems
                            .field(&format!("{name}.derivedFrom.version"), value.version)
                            .map(|value| value.as_str().to_owned()),
                    });
                Some(CohortFrequency {
                    derived_from,
                    focus_allele_count: problems.field(
                        &format!("{name}.focusAlleleCount"),
                        value.focus_allele_count,
                    ),
                    locus_allele_count: problems.field(
                        &format!("{name}.locusAlleleCount"),
                        value.locus_allele_count,
                    ),
                    cohort: super::Cohort {
                        id: value.cohort.id,
                    },
                })
            })
            .collect(),
    )
}
