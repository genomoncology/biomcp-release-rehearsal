use super::{SectionResult, SectionValue};
use crate::{Ctx, status};
use biodata::{Gene, HgncId};
use biomcp_adapters::clingen;
use biomcp_fetch::{Deadline, Leg, LegReport, RequestError};
use biomcp_types::{ClinGenGene, GeneSection, Section, SourceOutcome};

pub(super) fn run(cx: &Ctx<'_>, deadline: Deadline<'_>, genes: &[(HgncId, Gene)]) -> SectionResult {
    let leg = cx.fetch.leg(&clingen::SPEC, deadline).fresh(cx.fresh);
    let mut local = None;
    let mut dropped = false;
    let mut problems = Vec::new();
    let mut error = None;
    let mut report = LegReport::default();
    let values = genes
        .iter()
        .map(|(id, gene)| {
            let validity = clingen::validity(&leg, *id, &gene.symbol);
            let validity_error = validity.as_ref().err().copied();
            let validity_status = part_status(&leg, validity_error, &mut report);
            let dosage = clingen::dosage(&leg, *id, &gene.symbol);
            let dosage_error = dosage.as_ref().err().copied();
            let dosage_status = part_status(&leg, dosage_error, &mut report);
            error = dosage_error.or(validity_error).or(error);
            local = local
                .or(validity_error.and_then(status::local))
                .or(dosage_error.and_then(status::local));
            let dosage = dosage
                .map(|mapped| {
                    dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                    problems.extend(mapped.problems);
                    mapped.value
                })
                .unwrap_or_default();
            let validity = validity
                .map(|mapped| {
                    dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                    problems.extend(mapped.problems);
                    mapped.value
                })
                .unwrap_or_default();
            let data = ClinGenGene {
                validity_status,
                dosage_status,
                validity,
                haploinsufficiency: dosage
                    .as_ref()
                    .and_then(|row| row.haploinsufficiency.clone()),
                triplosensitivity: dosage
                    .as_ref()
                    .and_then(|row| row.triplosensitivity.clone()),
                dosage_date: dosage.and_then(|row| row.date),
            };
            (
                *id,
                SectionValue::ClinGen(Section::Data {
                    source: GeneSection::ClinGen.source(),
                    data,
                }),
            )
        })
        .collect();
    SectionResult {
        dropped,
        values,
        source: status::source(GeneSection::ClinGen.source(), report, error),
        problems,
        local,
    }
}

fn part_status(
    leg: &Leg<'_>,
    error: Option<RequestError>,
    combined: &mut LegReport,
) -> SourceOutcome {
    let report = leg.take_report();
    combined.stale |= report.stale;
    combined.failures.extend(report.failures.iter().copied());
    combined
        .cache_problems
        .extend(report.cache_problems.iter().copied());
    status::source(GeneSection::ClinGen.source(), report, error)
}
