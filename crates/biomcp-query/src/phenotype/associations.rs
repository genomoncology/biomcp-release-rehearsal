//! Exact direct source association pages, without joins or clinical inference.
use super::{empty, problem, recovery, run};
use crate::{Ctx, Outcome, status};
use biomcp_adapters::monarch;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiseaseGet, PageInfo, PhenotypeDiseases, PhenotypeGenes, PhenotypeGet, PhenotypeSection,
    ReplyData, Request, Source, StepKind,
};
pub(super) fn get(cx: &Ctx<'_>, q: &PhenotypeGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&monarch::SPEC, deadline).fresh(cx.fresh);
    let reply = monarch::phenotype::associations::get(&leg, q);
    let error = reply.as_ref().err().copied();
    let mut out = empty(body(q, Vec::new(), Box::default(), None));
    out.sources
        .push(status::source(Source::Monarch, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok(page) = reply {
        out.include_refusals(page.refusals);
        out.problems.extend(page.problems);
        let returned = page.items.len() as u32;
        out.page = Some(PageInfo {
            offset: q.section_offset(),
            returned,
            total: page.total,
            pool: None,
            truncated: None,
        });
        pivots(&mut out, q, &page.items);
        if page.received > 0
            && u64::from(q.section_offset()) + u64::from(page.received) < page.total
        {
            out.next.push(crate::next::Step::new(
                StepKind::Page,
                &[Source::Monarch],
                run(Request::GetPhenotype(q.at_section_offset(
                    q.section_offset().saturating_add(page.received),
                ))),
            ));
        }
        out.body = body(
            q,
            page.items,
            page.components,
            page.metadata
                .fetched
                .and_then(|value| u64::try_from(value).ok()),
        );
        {
            let absent = page.total == 0;
            crate::status::item(&mut out, !absent, false, false);
        }
    }
    problem(&mut out, Source::Monarch, q.id().as_str().parse().ok());
    recovery(&mut out, Request::GetPhenotype(q.clone()));
    command.finish(deadline.remaining());
    out
}
fn body(
    q: &PhenotypeGet,
    items: Vec<biomcp_types::DiseaseAssociationItem>,
    components: Box<[biomcp_types::ComponentLabel]>,
    fetched: Option<u64>,
) -> ReplyData {
    if q.section() == Some(PhenotypeSection::Genes) {
        (PhenotypeGenes {
            items: items.into(),
            components,
            fetched,
            direct: true,
        })
        .with_query(q.clone())
    } else {
        (PhenotypeDiseases {
            items: items.into(),
            components,
            fetched,
            direct: true,
        })
        .with_query(q.clone())
    }
}
fn pivots(out: &mut Outcome, q: &PhenotypeGet, rows: &[biomcp_types::DiseaseAssociationItem]) {
    for row in rows {
        let id = &row.association.subject.id;
        let id = id.to_string();
        let request = if q.section() == Some(PhenotypeSection::Genes) {
            crate::disease::monarch::gene_query(&id, &mut out.problems).map(Request::GetGene)
        } else {
            if biomcp_adapters::mydisease::supported_id(&id) {
                crate::next::checked(
                    id.parse::<biodata::Curie>(),
                    Source::Monarch,
                    &mut out.problems,
                )
                .and_then(|id| {
                    crate::next::checked(
                        DiseaseGet::from_id(id),
                        Source::Monarch,
                        &mut out.problems,
                    )
                })
                .map(Request::GetDisease)
            } else {
                None
            }
        };
        if let Some(request) = request {
            let step = crate::next::Step::new(
                StepKind::Pivot,
                &[if q.section() == Some(PhenotypeSection::Genes) {
                    Source::Hgnc
                } else {
                    Source::MyDisease
                }],
                run(request),
            );
            if !out.next.contains(&step) {
                out.next.push(step);
            }
        }
    }
}
