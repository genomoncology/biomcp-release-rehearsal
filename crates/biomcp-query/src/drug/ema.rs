use super::{empty, searched, unsupported};
use crate::{Action, Bundle, Ctx, Outcome, status};
use biomcp_adapters::ema;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugGet, DrugPage, DrugSearch, DrugSection, PageInfo, Request, Section, Source, SourceState,
    StepKind,
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
    let leg = cx.fetch.leg(&ema::SPEC, deadline).fresh(cx.fresh);
    let result = ema::search(&leg, name.as_str());
    let error = result.as_ref().err().copied();
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(Source::Ema, leg.finish(), error));
    if let Ok(rows) = result {
        out.include_refusals(rows.refusals);
        out.problems.extend(rows.problems);
        let rows = rows.value;
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
    let regulatory = query.sections() == [DrugSection::Regulatory];
    let mut out = empty(if regulatory {
        biomcp_types::DrugRegulatory::into_reply(query.clone(), None)
    } else {
        biomcp_types::DrugCard::into_reply(query.clone(), None)
    });
    if !query.supports_sections() {
        return unsupported(query.source(), out);
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&ema::SPEC, deadline).fresh(cx.fresh);
    let result = if regulatory {
        ema::regulatory(&leg, query.input()).map(|body| {
            out.include_refusals(body.refusals);
            out.problems.extend(body.problems);
            let body = body.value;
            let found = body.is_some();
            out.body = biomcp_types::DrugRegulatory::into_reply(
                query.clone(),
                body.map(|body| regulatory_page(query, body, &mut out)),
            );
            found
        })
    } else {
        ema::card(&leg, query.input()).map(|card| {
            out.include_refusals(card.refusals);
            out.problems.extend(card.problems);
            let card = card.value;
            if card.is_some()
                && let Some(get) = crate::next::checked(
                    query.for_section(DrugSection::Regulatory),
                    biomcp_types::Source::Ema,
                    &mut out.problems,
                )
            {
                out.next.push(crate::next::Step::run(
                    StepKind::Section,
                    &[biomcp_types::Source::Ema],
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
        .push(status::source(Source::Ema, leg.finish(), error));
    if let Ok(found) = result {
        {
            let (found, identity_failed, unavailable) = if !found {
                (false, false, false)
            } else if out.sources.iter().any(|s| s.state == SourceState::Stale) {
                (true, false, true)
            } else {
                (true, false, false)
            };
            crate::status::item(&mut out, found, identity_failed, unavailable);
        };
    }
    sync_step(&mut out);
    command.finish(deadline.remaining());
    out
}

fn regulatory_page(
    query: &DrugGet,
    mut body: biomcp_types::DrugRegulatory,
    out: &mut Outcome,
) -> biomcp_types::DrugRegulatory {
    if let Section::Data { data, .. } = &mut body.regulatory {
        let offset = query.section_offset().unwrap_or(0);
        let total = data.records.len() as u64;
        data.records = data
            .records
            .iter()
            .skip(offset as usize)
            .take(query.section_limit().unwrap_or(10) as usize)
            .cloned()
            .collect();
        data.offset = offset;
        out.page = Some(PageInfo {
            offset,
            returned: data.records.len() as u32,
            total,
            pool: None,
            truncated: None,
        });
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
            &[biomcp_types::Source::Ema],
            Action::Sync {
                bundle: Bundle::EMA,
            },
        ));
    }
}
