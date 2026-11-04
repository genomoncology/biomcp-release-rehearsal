use super::page::{empty, page_info, run, slice};
use crate::{Ctx, Outcome, status};
use biomcp_adapters::ols;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiseaseGet, DiseasePage, DiseaseSearch, DiseaseSection, DiseaseSynonyms, DiseaseXrefs,
    ReplyData, Request, Source, StepKind,
};
/// Discover one source-ordered page from one explicit ontology.
pub(crate) fn search(cx: &Ctx<'_>, query: &DiseaseSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&ols::SPEC, deadline).fresh(cx.fresh);
    let reply = ols::search(&leg, query);
    let error = reply.as_ref().err().copied();
    let mut out = empty(
        (DiseasePage {
            items: Box::default(),
            components: query
                .ontology()
                .and_then(|v| ols::components(v).ok())
                .unwrap_or_default(),
        })
        .with_query(query.clone()),
    );
    out.sources
        .push(status::source(Source::Ols, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok(page) = reply {
        out.include_refusals(page.rows.refusals);
        let returned = page.rows.value.len() as u32;
        for row in &page.rows.value {
            if let Some(id) = &row.concept.id
                && let Some(get) = crate::next::checked(
                    id.to_string()
                        .parse()
                        .map_err(|_| biomcp_types::QueryInvalid::new(&["id"]))
                        .and_then(|id| DiseaseGet::ols(id, row.ontology)),
                    Source::Ols,
                    &mut out.problems,
                )
            {
                out.next.push(crate::next::Step::new(
                    StepKind::Disambiguate,
                    &[biomcp_types::Source::Ols],
                    run(Request::GetDisease(get)),
                ));
            }
        }
        if page.received > 0 && u64::from(query.offset()) + u64::from(page.received) < page.total {
            out.next.push(crate::next::Step::new(
                StepKind::Page,
                &[biomcp_types::Source::Ols],
                run(Request::SearchDisease(
                    query.at_offset(query.offset().saturating_add(page.received)),
                )),
            ));
        }
        out.page = Some(page_info(query.offset(), returned, page.total));
        out.problems.extend(page.rows.problems);
        if let ReplyData::DiseasePage {
            body: Some(body), ..
        } = &mut out.body
        {
            body.items = page.rows.value.into();
        }
        crate::status::search(&mut out);
    }
    command.finish(deadline.remaining());
    out
}
/// Get an exact OLS record; section slicing reuses the checked original response.
pub(crate) fn get(cx: &Ctx<'_>, query: &DiseaseGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&ols::SPEC, deadline).fresh(cx.fresh);
    let reply = ols::get(&leg, query);
    let error = reply.as_ref().err().copied();
    let mut out = empty(biomcp_types::DiseaseCard::into_reply(query.clone(), None));
    out.sources
        .push(status::source(Source::Ols, leg.finish(), error));
    out.local = error.and_then(status::local);
    match reply {
        Ok(Some(record)) => apply(&mut out, query, record),
        Ok(None) => crate::status::item(&mut out, false, false, false),
        _ => (),
    }
    super::page::record_problem(&mut out, query, Source::Ols);
    command.finish(deadline.remaining());
    out
}
fn apply(out: &mut Outcome, query: &DiseaseGet, record: ols::Record) {
    let components = record.card.components.clone();
    out.problems = record.problems;
    match query.section() {
        Some(DiseaseSection::Synonyms) => {
            out.include_refusals(record.synonyms.refusals);
            out.problems.extend(record.synonyms.problems);
            let (items, total) = slice(record.synonyms.value, query);
            out.page = Some(page_info(query.section_offset(), items.len() as u32, total));
            out.body = (DiseaseSynonyms {
                items: items.into(),
                components,
            })
            .with_query(query.clone());
        }
        Some(DiseaseSection::Xrefs) => {
            out.include_refusals(record.xrefs.refusals);
            out.problems.extend(record.xrefs.problems);
            let (items, total) = slice(record.xrefs.value, query);
            out.page = Some(page_info(query.section_offset(), items.len() as u32, total));
            out.body = (DiseaseXrefs {
                items: items.into(),
                components,
            })
            .with_query(query.clone());
        }
        _ => {
            out.include_refusals(record.synonyms.refusals);
            out.problems.extend(record.synonyms.problems);
            out.include_refusals(record.xrefs.refusals);
            out.problems.extend(record.xrefs.problems);
            for section in record.card.sections.iter() {
                out.next.push(crate::next::Step::new(
                    StepKind::Section,
                    &[biomcp_types::Source::Ols],
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
                &[biomcp_types::Source::Ols],
                run(Request::GetDisease(query.at_section_offset(offset))),
            ));
        }
    }
    crate::status::item(out, true, false, false);
}
