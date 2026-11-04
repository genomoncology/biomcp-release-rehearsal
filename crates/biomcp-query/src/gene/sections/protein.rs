use super::{SectionResult, SectionValue};
use crate::{Ctx, status};
use biodata::{Gene, HgncId};

use biomcp_adapters::uniprot;
use biomcp_fetch::Deadline;
use biomcp_types::{GeneSection, Section};
pub(super) fn applicable(gene: &Gene) -> bool {
    !gene.uniprot.is_empty()
}
pub(super) fn run(cx: &Ctx<'_>, deadline: Deadline<'_>, genes: &[(HgncId, Gene)]) -> SectionResult {
    let ids: Vec<_> = genes
        .iter()
        .flat_map(|(_, gene)| gene.uniprot.iter().cloned())
        .collect();
    let leg = cx.fetch.leg(&uniprot::SPEC, deadline).fresh(cx.fresh);
    let result = uniprot::proteins(&leg, &ids);
    let error = result.as_ref().err().copied();
    let source = status::source(GeneSection::Protein.source(), leg.finish(), error);
    let mut dropped = false;
    let mut problems = Vec::new();
    if let Ok(mapped) = &result {
        dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
        status::append(&mut problems, mapped.problems.clone());
    }
    let values = genes
        .iter()
        .filter(|(_, gene)| applicable(gene))
        .map(|(id, gene)| {
            let section = match &result {
                Ok(mapped) => {
                    let data: Vec<_> = gene
                        .uniprot
                        .iter()
                        .filter_map(|id| {
                            mapped
                                .value
                                .iter()
                                .find(|protein| protein.accession == *id)
                                .cloned()
                        })
                        .collect();
                    if data.is_empty() {
                        Section::Empty {
                            source: GeneSection::Protein.source(),
                            record: None,
                        }
                    } else {
                        Section::Data {
                            source: GeneSection::Protein.source(),
                            data: data.into_boxed_slice(),
                        }
                    }
                }
                Err(error) => Section::unavailable(
                    GeneSection::Protein.source(),
                    crate::status::reason(*error).unwrap_or(biomcp_types::Reason::Unavailable),
                ),
            };
            (*id, SectionValue::Protein(section))
        })
        .collect();
    SectionResult {
        dropped,
        values,
        source,
        problems,
        local: error.and_then(status::local),
    }
}
