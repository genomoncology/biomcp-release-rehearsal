use super::{SectionResult, SectionValue};
use crate::{Ctx, status};
use biodata::{Gene, HgncId};

use biomcp_adapters::reactome;
use biomcp_fetch::Deadline;
use biomcp_types::{Capped, GeneSection, Section};
const LIMIT: usize = 25;
pub(super) fn applicable(gene: &Gene) -> bool {
    !gene.uniprot.is_empty()
}
pub(super) fn run(cx: &Ctx<'_>, deadline: Deadline<'_>, genes: &[(HgncId, Gene)]) -> SectionResult {
    let mut ids = Vec::new();
    for (_, gene) in genes {
        for id in &gene.uniprot {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
    }
    let leg = cx.fetch.leg(&reactome::SPEC, deadline).fresh(cx.fresh);
    let results = leg.batch(&ids, reactome::pathways);
    let error = results
        .iter()
        .find_map(|result| result.as_ref().err().copied());
    let local = results
        .iter()
        .find_map(|result| result.as_ref().err().copied().and_then(status::local));
    let source = status::source(GeneSection::Pathways.source(), leg.finish(), error);
    let mut dropped = false;
    let mut problems = Vec::new();
    let mut values = Vec::new();
    for (id, gene) in genes.iter().filter(|(_, gene)| applicable(gene)) {
        let mut pathways = Vec::new();
        let mut failure = None;
        for accession in &gene.uniprot {
            let Some(index) = ids.iter().position(|id| id == accession) else {
                continue;
            };
            let Some(result) = results.get(index) else {
                continue;
            };
            match result {
                Ok(mapped) => {
                    dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                    status::append(&mut problems, mapped.problems.clone());
                    for pathway in &mapped.value {
                        if !pathways
                            .iter()
                            .any(|item: &biomcp_types::PathwayRef| item.id == pathway.id)
                        {
                            pathways.push(pathway.clone());
                        }
                    }
                }
                Err(error) => {
                    failure.get_or_insert(
                        crate::status::reason(*error).unwrap_or(biomcp_types::Reason::Unavailable),
                    );
                }
            }
        }
        let section = if let Some(reason) = failure {
            Section::unavailable(GeneSection::Pathways.source(), reason)
        } else if pathways.is_empty() {
            Section::Empty {
                source: GeneSection::Pathways.source(),
                record: None,
            }
        } else {
            let total = u64::try_from(pathways.len()).unwrap_or(u64::MAX);
            pathways.truncate(LIMIT);
            Section::Data {
                source: GeneSection::Pathways.source(),
                data: Capped {
                    items: pathways.into(),
                    total,
                },
            }
        };
        values.push((*id, SectionValue::Pathways(section)));
    }
    SectionResult {
        dropped,
        values,
        source,
        problems,
        local,
    }
}
