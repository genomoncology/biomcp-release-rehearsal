use super::{SectionResult, SectionValue};
use crate::{Ctx, status};
use biodata::{Gene, HgncId};

use biomcp_adapters::gencc;
use biomcp_fetch::Deadline;
use biomcp_types::{GeneSection, Reason, Section};
pub(super) fn run(cx: &Ctx<'_>, deadline: Deadline<'_>, genes: &[(HgncId, Gene)]) -> SectionResult {
    let leg = cx.fetch.leg(&gencc::SPEC, deadline).fresh(cx.fresh);
    let mut error = None;
    let mut dropped = false;
    let mut problems = Vec::new();
    let mut local = None;
    let values = genes
        .iter()
        .map(|(id, gene)| {
            let section = match gencc::gene(&leg, *id, &gene.symbol) {
                Ok(mapped) => {
                    dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                    status::append(&mut problems, mapped.problems);
                    match mapped.value {
                        Some(data) => Section::Data {
                            source: GeneSection::Gencc.source(),
                            data,
                        },
                        None => Section::Empty {
                            source: GeneSection::Gencc.source(),
                            record: None,
                        },
                    }
                }
                Err(failure) => {
                    error = Some(failure);
                    local = local.or(status::local(failure));
                    Section::unavailable(
                        GeneSection::Gencc.source(),
                        crate::status::reason(failure).unwrap_or(biomcp_types::Reason::Unavailable),
                    )
                }
            };
            (*id, SectionValue::Gencc(section))
        })
        .collect();
    let report = leg.finish();
    let mut values: Vec<_> = values;
    if !report.failures.is_empty() {
        let reason = report
            .failures
            .first()
            .map_or(Reason::Unavailable, |e| e.reason);
        for (_, value) in &mut values {
            *value = SectionValue::Gencc(Section::unavailable(GeneSection::Gencc.source(), reason));
        }
    }
    SectionResult {
        dropped,
        values,
        source: status::source(GeneSection::Gencc.source(), report, error),
        problems,
        local,
    }
}
