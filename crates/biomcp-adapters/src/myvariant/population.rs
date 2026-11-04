//! Candidate-bound cross-reference joins; no local coordinate conversion.
//! Source fields: https://biothings.io/blog/new-data-release-201606
//! CA identifier contract: https://reg.genome.network/doc/alleleTerminology.html
use super::raw::Hit;
use crate::{Mapped, raw::Field};
use biodata::VariantId;
use biomcp_fetch::Leg;
use biomcp_types::{Source, VariantHeader};

pub(super) fn fields(leg: &Leg<'_>) -> String {
    let mut fields = super::fields(leg);
    fields.push_str(",chrom,hg19,hg38");
    if !leg.disabled_source(Source::AlleleRegistry) {
        fields.push_str(",clingen.caid");
    }
    fields
}
pub(super) fn candidates(
    leg: &Leg<'_>,
    hits: Vec<Field<Hit>>,
    query: &str,
) -> Mapped<Vec<VariantHeader>> {
    let mut result = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    for raw in hits {
        let Some(mut hit) = raw.value() else {
            result.refusals = crate::Refusal::Refused;
            result
                .problems
                .push(crate::mapped::problem(Source::MyVariant, None, "identity"));
            continue;
        };
        let ca = if leg.disabled_source(Source::AlleleRegistry) {
            None
        } else {
            hit.clingen.take()
        };
        let build = super::search_build::source_build(&hit);
        let mut mapped = super::map::candidates(
            vec![Field::Value(hit)],
            query,
            build,
            !super::components::disabled_on_leg(leg, "clinvar"),
            !super::components::disabled_on_leg(leg, "snpeff"),
        );
        if let Some(row) = mapped.value.first_mut() {
            match ca {
                Some(Field::Value(source)) => match source
                    .caid
                    .value()
                    .and_then(|id| id.as_str().parse::<VariantId>().ok())
                {
                    Some(id @ VariantId::Ca(_)) => {
                        let mut ids = row.identity.ids.to_vec();
                        ids.push(id);
                        row.identity.ids = ids.into();
                    }
                    _ => mapped.problems.push(crate::mapped::problem(
                        Source::MyVariant,
                        Some(row.record.id.clone()),
                        "clingen.caid",
                    )),
                },
                Some(Field::Invalid(_)) => mapped.problems.push(crate::mapped::problem(
                    Source::MyVariant,
                    Some(row.record.id.clone()),
                    "clingen.caid",
                )),
                None => (),
            }
        }
        result.refusals.include(mapped.refusals);
        result.value.extend(mapped.value);
        result.problems.extend(mapped.problems);
    }
    result
}
