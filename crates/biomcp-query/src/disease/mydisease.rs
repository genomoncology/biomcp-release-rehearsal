use super::page::{empty, page_info, run, slice};
use crate::{Ctx, Outcome, status};
use biomcp_adapters::mydisease;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiseaseGet, DiseaseHierarchy, DiseaseSection, DiseaseSynonyms, DiseaseXrefs, Request, Source,
    StepKind,
};

/// Read only the selected exact annotation, then page its native component facts.
pub(crate) fn get(cx: &Ctx<'_>, query: &DiseaseGet, id: &biodata::Curie) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&mydisease::SPEC, deadline).fresh(cx.fresh);
    let reply = mydisease::annotation(&leg, id);
    let error = reply.as_ref().err().copied();
    let mut out = empty(biomcp_types::DiseaseCard::into_reply(query.clone(), None));
    out.sources
        .push(status::source(Source::MyDisease, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok(mapped) = reply {
        out.include_refusals(mapped.refusals);
        out.problems = mapped.problems;
        apply(&mut out, query, mapped.value);
    }
    super::page::record_problem(&mut out, query, Source::MyDisease);
    command.finish(deadline.remaining());
    out
}
fn apply(out: &mut Outcome, query: &DiseaseGet, record: mydisease::Annotation) {
    let components = record.card.components.clone();
    match query.section() {
        Some(DiseaseSection::Synonyms) => {
            let (items, total) = slice(record.synonyms, query);
            out.page = Some(page_info(query.section_offset(), items.len() as u32, total));
            out.body = (DiseaseSynonyms {
                items: items.into(),
                components,
            })
            .with_query(query.clone());
        }
        Some(DiseaseSection::Xrefs) => {
            let (items, total) = slice(record.xrefs, query);
            out.page = Some(page_info(query.section_offset(), items.len() as u32, total));
            out.body = (DiseaseXrefs {
                items: items.into(),
                components,
            })
            .with_query(query.clone());
        }
        Some(DiseaseSection::Hierarchy) => {
            let (items, total) = slice(record.parents, query);
            out.page = Some(page_info(query.section_offset(), items.len() as u32, total));
            out.body = (DiseaseHierarchy {
                items: items.into(),
                components,
            })
            .with_query(query.clone());
        }
        _ => {
            for section in &record.card.sections {
                out.next.push(crate::next::Step::new(
                    StepKind::Section,
                    &[biomcp_types::Source::MyDisease],
                    run(Request::GetDisease(query.with_section(*section))),
                ));
            }
            out.body = (record.card).with_query(query.clone());
        }
    }
    if let Some(page) = &out.page {
        let offset = page.offset.saturating_add(page.returned);
        if page.returned > 0 && u64::from(offset) < page.total {
            out.next.push(crate::next::Step::new(
                StepKind::Page,
                &[biomcp_types::Source::MyDisease],
                run(Request::GetDisease(query.at_section_offset(offset))),
            ));
        }
    }
    crate::status::item(out, true, false, false);
}
