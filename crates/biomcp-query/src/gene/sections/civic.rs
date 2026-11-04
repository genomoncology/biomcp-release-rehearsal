use super::{SectionResult, SectionValue};
use crate::{Ctx, status};
use biodata::{Gene, HgncId, Text};

use biomcp_adapters::civic;
use biomcp_fetch::Deadline;
use biomcp_types::{GeneSection, Problem, ProblemCode, Reason, Section, Source};
pub(super) fn run(cx: &Ctx<'_>, deadline: Deadline<'_>, genes: &[(HgncId, Gene)]) -> SectionResult {
    let ids: Vec<_> = genes
        .iter()
        .filter_map(|(_, gene)| gene.entrez_id.map(|id| (gene.symbol.clone(), id)))
        .collect();
    let leg = cx.fetch.leg(&civic::SPEC, deadline).fresh(cx.fresh);
    let result = if ids.is_empty() {
        Ok(Vec::new())
    } else {
        civic::genes(&leg, &ids)
    };
    let error = result.as_ref().err().copied();
    let mut source = status::source(GeneSection::Civic.source(), leg.finish(), error);
    let mut dropped = false;
    let mut problems = Vec::new();
    let values = genes
        .iter()
        .map(|(id, gene)| {
            let section = if let Some(entrez) = gene.entrez_id {
                match &result {
                    Ok(values) => match values
                        .iter()
                        .find(|value| value.id == entrez && value.symbol == gene.symbol)
                    {
                        Some(value) => {
                            dropped |= value.civic.refusals == biomcp_adapters::Refusal::Refused;
                            status::append(&mut problems, value.civic.problems.clone());
                            value.civic.value.clone()
                        }
                        None => Section::unavailable(GeneSection::Civic.source(), Reason::BadReply),
                    },
                    Err(error) => Section::unavailable(
                        GeneSection::Civic.source(),
                        crate::status::reason(*error).unwrap_or(biomcp_types::Reason::Unavailable),
                    ),
                }
            } else {
                problems.push(Problem {
                    source: Source::Hgnc,
                    record: Text::new(&id.to_string()).ok(),
                    field: "entrez_id".into(),
                    code: ProblemCode::ValueUnknown,
                });
                Section::unavailable(GeneSection::Civic.source(), Reason::NoJoinId)
            };
            (*id, SectionValue::Civic(section))
        })
        .collect();
    if ids.is_empty() {
        source.state = biomcp_types::SourceState::Failed;
        source.reason = Some(Reason::NoJoinId);
    }
    SectionResult {
        dropped,
        values,
        source,
        problems,
        local: error.and_then(status::local),
    }
}
