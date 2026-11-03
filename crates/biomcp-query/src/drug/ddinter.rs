use super::{empty, searched, unsupported};
use crate::{Action, Bundle, Ctx, Outcome, status};
use biomcp_adapters::ddinter;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugGet, DrugInteractions, DrugPage, DrugSearch, DrugSection, PageInfo, Request, Section,
    Source, SourceState, StepKind,
};

pub(super) fn search(cx: &Ctx<'_>, query: &DrugSearch) -> Outcome {
    let mut out = empty(
        (DrugPage {
            items: Box::default(),
            total: None,
        })
        .with_query(query.clone()),
    );
    let Some(name) = query.query() else {
        return super::unsupported(query.source(), out);
    };
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&ddinter::SPEC, deadline).fresh(cx.fresh);
    let result = ddinter::search(&leg, name.as_str());
    let error = result.as_ref().err().copied();
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(Source::Ddinter, leg.finish(), error));
    if let Ok(mapped) = result {
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        let rows = mapped.value;
        let total = rows.len() as u64;
        let page = DrugPage {
            items: rows
                .into_iter()
                .skip(query.offset() as usize)
                .take(query.limit() as usize)
                .collect(),
            total: Some(total),
        };
        searched(query, cx.fresh, page, &mut out);
    }
    sync_step(&mut out);
    command.finish(deadline.remaining());
    out
}

pub(super) fn get(cx: &Ctx<'_>, query: &DrugGet) -> Outcome {
    let interactions = query.sections() == [DrugSection::Interactions];
    let mut out = empty(if interactions {
        biomcp_types::DrugInteractions::into_reply(query.clone(), None)
    } else {
        biomcp_types::DrugCard::into_reply(query.clone(), None)
    });
    if !query.supports_sections() {
        return unsupported(query.source(), out);
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&ddinter::SPEC, deadline).fresh(cx.fresh);
    let result = if interactions {
        ddinter::interactions(
            &leg,
            query.input(),
            query.section_limit().unwrap_or(5),
            query.section_offset().unwrap_or(0),
        )
        .map(|body| {
            out.include_refusals(body.refusals);
            out.problems.extend(body.problems);
            let body = body.value;
            let found = body.is_some();
            out.body = biomcp_types::DrugInteractions::into_reply(
                query.clone(),
                body.map(|body| page(query, cx.fresh, body, &mut out)),
            );
            found
        })
    } else {
        ddinter::card(&leg, query.input()).map(|card| {
            out.include_refusals(card.refusals);
            out.problems.extend(card.problems);
            let card = card.value;
            if card.is_some()
                && let Some(get) = crate::next::checked(
                    query.for_section(DrugSection::Interactions),
                    biomcp_types::Source::Ddinter,
                    &mut out.problems,
                )
            {
                out.next.push(crate::next::Step::run(
                    StepKind::Section,
                    &[biomcp_types::Source::Ddinter],
                    Request::GetDrug(get),
                    cx.fresh,
                ));
            }
            let found = card.is_some();
            out.body = biomcp_types::DrugCard::into_reply(query.clone(), card);
            found
        })
    };
    let error = result.as_ref().err().copied();
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(Source::Ddinter, leg.finish(), error));
    if let Ok(found) = result {
        status::item(&mut out, found, false, false);
    }
    sync_step(&mut out);
    command.finish(deadline.remaining());
    out
}
fn page(
    query: &DrugGet,
    fresh: bool,
    body: DrugInteractions,
    out: &mut Outcome,
) -> DrugInteractions {
    if let Section::Data { data, .. } = &body.interactions {
        let returned = data.records.len() as u32;
        let total = data.total.unwrap_or(0);
        out.page = Some(PageInfo {
            offset: data.offset,
            returned,
            total,
            pool: None,
            truncated: None,
        });
        if let Some(offset) = data
            .offset
            .checked_add(returned)
            .filter(|n| returned > 0 && u64::from(*n) < total)
            && let Some(get) = crate::next::checked(
                query.at_section_offset(offset),
                biomcp_types::Source::Ddinter,
                &mut out.problems,
            )
        {
            out.next.push(crate::next::Step::run(
                StepKind::Page,
                &[biomcp_types::Source::Ddinter],
                Request::GetDrug(get),
                fresh,
            ));
        }
    }
    body
}
fn sync_step(out: &mut Outcome) {
    if out
        .sources
        .iter()
        .any(|s| matches!(s.state, SourceState::Syncing | SourceState::Stale))
    {
        out.next.push(crate::next::Step::new(
            StepKind::Sync,
            &[biomcp_types::Source::Ddinter],
            Action::Sync {
                bundle: Bundle::DDINTER,
            },
        ));
    }
}
