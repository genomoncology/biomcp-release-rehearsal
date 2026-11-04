use super::SectionResult;
use crate::{Ctx, status};
use biodata::{Gene, HgncId, LocusGroup};

use biomcp_adapters::gnomad;
use biomcp_fetch::Deadline;
use biomcp_types::{GeneSection, Reason, Section};

pub(super) fn applicable(gene: &Gene) -> bool {
    gene.locus_group == Some(LocusGroup::ProteinCoding) && gene.ensembl_id.is_some()
}
pub(super) fn run(cx: &Ctx<'_>, deadline: Deadline<'_>, genes: &[(HgncId, Gene)]) -> SectionResult {
    let ids: Vec<_> = genes
        .iter()
        .filter(|(_, gene)| applicable(gene))
        .filter_map(|(_, gene)| gene.ensembl_id.clone())
        .collect();
    let leg = cx.fetch.leg(&gnomad::SPEC, deadline).fresh(cx.fresh);
    let result = gnomad::constraints(&leg, &ids);
    let error = result.as_ref().err().copied();
    let source = status::source(GeneSection::Constraint.source(), leg.finish(), error);
    let mut dropped = false;
    let mut problems = Vec::new();
    let mut sections = Vec::new();
    for (id, gene) in genes.iter().filter(|(_, gene)| applicable(gene)) {
        let section = match &result {
            Ok(values) => {
                let value = values.iter().find(|value| {
                    gene.ensembl_id
                        .as_ref()
                        .is_some_and(|id| id.base() == value.id.base())
                });
                match value {
                    Some(value) => {
                        dropped |= value.constraint.refusals == biomcp_adapters::Refusal::Refused;
                        status::append(&mut problems, value.constraint.problems.clone());
                        if value
                            .constraint
                            .problems
                            .iter()
                            .any(|problem| problem.field.as_ref() == "gene_id")
                        {
                            Section::unavailable(GeneSection::Constraint.source(), Reason::BadReply)
                        } else if let Some(data) = &value.constraint.value {
                            Section::Data {
                                source: GeneSection::Constraint.source(),
                                data: data.clone(),
                            }
                        } else {
                            Section::Empty {
                                source: GeneSection::Constraint.source(),
                                record: None,
                            }
                        }
                    }
                    None => {
                        Section::unavailable(GeneSection::Constraint.source(), Reason::BadReply)
                    }
                }
            }
            Err(error) => Section::unavailable(
                GeneSection::Constraint.source(),
                crate::status::reason(*error).unwrap_or(biomcp_types::Reason::Unavailable),
            ),
        };
        sections.push((*id, super::SectionValue::Constraint(section)));
    }
    SectionResult {
        dropped,
        values: sections,
        source,
        problems,
        local: error.and_then(status::local),
    }
}
