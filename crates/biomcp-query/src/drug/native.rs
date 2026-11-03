//! Source-local native drug requests; selected identities never fan out.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod identity;
mod sections;
use crate::{Ctx, Outcome, status};
use biomcp_adapters::{NativeWindow, chembl, opentargets};
use biomcp_fetch::{Leg, RequestError, SourceError};
use biomcp_types::{
    DrugGet, DrugPage, DrugSearch, DrugSection, DrugSource, PageInfo, Reason, Request, Source,
    Status, StepKind,
};
fn failure(source: Source) -> RequestError {
    RequestError::Source(SourceError {
        source,
        reason: Reason::BadReply,
        status: None,
        retry_after: None,
    })
}
pub(super) use crate::status::empty;
fn spec(source: DrugSource) -> &'static biomcp_fetch::SourceSpec {
    if source == DrugSource::Chembl {
        &chembl::SPEC
    } else {
        &opentargets::SPEC
    }
}
pub(super) fn search(cx: &Ctx<'_>, query: &DrugSearch) -> Outcome {
    let mut out = empty(
        (DrugPage {
            items: Box::default(),
            total: None,
        })
        .with_query(query.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(spec(query.source()), deadline).fresh(cx.fresh);
    let result = search_page(&leg, query, &mut out.problems);
    let error = result.as_ref().err().copied();
    if let Ok(window) = result {
        out.include_refusals(window.refusals());
        let next_offset = window.next_offset().filter(|_| window.total().is_some());
        let page = DrugPage {
            total: window.total(),
            items: window.rows.into(),
        };
        let returned = page.items.len() as u32;
        out.page = page.total.map(|total| PageInfo {
            offset: query.offset(),
            returned,
            total,
            pool: None,
            truncated: None,
        });
        for item in &page.items {
            if let Some(get) = crate::next::checked(
                DrugGet::new(item.id.clone(), query.source()),
                query.source().source(),
                &mut out.problems,
            ) {
                out.next.push(crate::next::Step::run(
                    StepKind::Disambiguate,
                    &[get.source().source()],
                    Request::GetDrug(get),
                    cx.fresh,
                ));
            }
        }
        if let Some(offset) = next_offset {
            out.next.push(crate::next::Step::run(
                StepKind::Page,
                &[query.source().source()],
                Request::SearchDrug(query.at_offset(offset)),
                cx.fresh,
            ));
        }
        out.body = page.with_query(query.clone());
    }
    finish(&mut out, leg, query.source().source(), error);
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
fn search_page(
    leg: &Leg<'_>,
    query: &DrugSearch,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Result<NativeWindow<biomcp_types::DrugHeader>, RequestError> {
    let name = query
        .query()
        .ok_or(biomcp_fetch::InputError::Serialization)?;
    let source = query.source().source();
    if query.source() == DrugSource::Chembl {
        let (page, _) = chembl::search(
            leg,
            name.as_str(),
            query.name_selector(),
            query.limit(),
            query.offset(),
        )?;
        problems.extend(page.problems);
        let items = page
            .window
            .rows
            .iter()
            .map(|row| identity::chembl_header(row, problems))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| failure(source))?;
        Ok(page.window.with_rows(items))
    } else {
        let (page, _) = opentargets::search(
            leg,
            name.as_str(),
            query.offset() / query.limit(),
            query.limit(),
        )?;
        let mapped = opentargets::search_headers(page, (query.offset() % query.limit()) as usize);
        problems.extend(mapped.problems);
        let received = mapped.value.items.len() as u32;
        Ok(NativeWindow::new(
            mapped.value.items.into_vec(),
            query.offset(),
            query.limit(),
            mapped.value.total,
            received,
            mapped.refusals,
        ))
    }
}

pub(super) fn get(cx: &Ctx<'_>, query: &DrugGet) -> Outcome {
    let mut out = empty(match query.sections() {
        [DrugSection::Targets] => biomcp_types::DrugTargets::into_reply(query.clone(), None),
        [DrugSection::Indications] => {
            biomcp_types::DrugIndications::into_reply(query.clone(), None)
        }
        [DrugSection::Xrefs] => biomcp_types::DrugXrefs::into_reply(query.clone(), None),
        _ => biomcp_types::DrugCard::into_reply(query.clone(), None),
    });
    if !query.supports_sections() {
        return super::unsupported(query.source(), out);
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(spec(query.source()), deadline).fresh(cx.fresh);
    let result = if query.source() == DrugSource::Chembl {
        sections::chembl(&leg, query, cx.fresh, &mut out)
    } else {
        sections::opentargets(&leg, query, cx.fresh, &mut out)
    };
    let error = result.err();
    if error.is_some() {
        crate::status::item(&mut out, false, true, false);
    }
    finish(&mut out, leg, query.source().source(), error);
    command.finish(deadline.remaining());
    out
}
fn finish(out: &mut Outcome, leg: Leg<'_>, source: Source, error: Option<RequestError>) {
    out.local = error.and_then(status::local);
    let mut outcome = status::source(source, leg.finish(), error);
    if let Some(page) = &out.page {
        outcome.count = Some(page.total);
        outcome.retrieved_count = Some(page.returned);
    }
    out.sources.push(outcome);
    if out.status() == Status::Complete {
        status::item(out, true, false, false);
    }
}
