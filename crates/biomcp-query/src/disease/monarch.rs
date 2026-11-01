use super::page::{empty, page_info, run};
use crate::{Ctx, Outcome, status};
use biomcp_adapters::monarch;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiseaseGenes, DiseaseGet, DiseasePhenotypes, DiseaseSection, GeneGet, GeneInput, ReplyData,
    Request, Source, StepKind,
};
/// Read one native association page without identity choice or cross-source joins.
pub(crate) fn get(cx: &Ctx<'_>, query: &DiseaseGet, id: &biodata::Curie) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&monarch::SPEC, deadline).fresh(cx.fresh);
    let reply = monarch::associations(&leg, query, id);
    let error = reply.as_ref().err().copied();
    let mut out = empty(body(query, Vec::new(), Box::default(), None));
    out.sources
        .push(status::source(Source::Monarch, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok(page) = reply {
        out.include_refusals(page.refusals);
        out.problems.extend(page.problems);
        let returned = page.items.len() as u32;
        out.page = Some(page_info(query.section_offset(), returned, page.total));
        if query.section() == Some(DiseaseSection::Genes) {
            gene_steps(&mut out, &page.items);
        }
        if page.received > 0
            && u64::from(query.section_offset()) + u64::from(page.received) < page.total
        {
            out.next.push(crate::next::Step::new(
                StepKind::Page,
                &[biomcp_types::Source::Monarch],
                run(Request::GetDisease(query.at_section_offset(
                    query.section_offset().saturating_add(page.received),
                ))),
            ));
        }
        let components = page.components;
        out.body = body(
            query,
            page.items,
            components,
            page.metadata
                .fetched
                .and_then(|value| u64::try_from(value).ok()),
        );
        {
            let absent = page.total == 0;
            crate::status::item(&mut out, !absent, false, false);
        }
    }
    super::page::record_problem(&mut out, query, Source::Monarch);
    command.finish(deadline.remaining());
    out
}
fn body(
    query: &DiseaseGet,
    items: Vec<biomcp_types::DiseaseAssociationItem>,
    components: Box<[biomcp_types::ComponentLabel]>,
    fetched: Option<u64>,
) -> ReplyData {
    if query.section() == Some(DiseaseSection::Genes) {
        (DiseaseGenes {
            items: items.into(),
            components,
            fetched,
        })
        .with_query(query.clone())
    } else {
        (DiseasePhenotypes {
            items: items.into(),
            components,
            fetched,
        })
        .with_query(query.clone())
    }
}
fn gene_steps(out: &mut Outcome, items: &[biomcp_types::DiseaseAssociationItem]) {
    for row in items {
        let id = &row.association.subject.id;
        let id = id.to_string();
        if let Some(query) = gene_query(&id, &mut out.problems) {
            let step = crate::next::Step::new(
                StepKind::Disambiguate,
                &[biomcp_types::Source::Hgnc],
                run(Request::GetGene(query)),
            );
            if !out.next.contains(&step) {
                out.next.push(step);
            }
        }
    }
}

/// Accept only an identifier the existing gene command understands, without symbol repair.
pub(crate) fn gene_query(id: &str, problems: &mut Vec<biomcp_types::Problem>) -> Option<GeneGet> {
    let input = if let Some(value) = id.strip_prefix("NCBIGene:") {
        crate::next::checked(
            value.parse::<biodata::NcbiGeneId>(),
            Source::Monarch,
            problems,
        )
        .map(GeneInput::EntrezId)?
    } else if id.starts_with("HGNC:")
        || id.starts_with("ENSG")
        || id.bytes().all(|byte| byte.is_ascii_digit())
    {
        crate::next::checked(GeneInput::new(id), Source::Monarch, problems)?
    } else {
        return None;
    };
    crate::next::checked(GeneGet::new(vec![input].into()), Source::Monarch, problems)
}
