use super::{empty, next};
use crate::{Ctx, Outcome, status};
use biomcp_adapters::fda_device_recall as adapter;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiagnosticCard, DiagnosticCountUnit, DiagnosticGet, DiagnosticPage, DiagnosticSearch, PageInfo,
    Problem, ProblemCode, Request, Source, SourceState, Status, StepKind,
};
pub(super) fn search(cx: &Ctx<'_>, q: &DiagnosticSearch) -> Outcome {
    let mut out = empty(
        (DiagnosticPage {
            items: Box::default(),
            count_unit: Some(DiagnosticCountUnit::RecallRecords),
            distinct_id_total: None,
        })
        .with_query(q.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&adapter::SPEC, deadline).fresh(cx.fresh);
    let result = adapter::search(&leg, q);
    let error = result.as_ref().err().copied();
    if let Ok(page) = result {
        let returned = page.items.len() as u32;
        {
            let found = page.total != 0;
            crate::status::item(&mut out, found, false, false);
        };
        out.problems = page.problems;
        next::candidates(&mut out, q, &page.items, cx.fresh);
        if u64::from(q.offset()) + u64::from(returned) < page.total && returned > 0 {
            next::push(
                &mut out,
                StepKind::Page,
                &[q.source().source()],
                Request::SearchDiagnostic(q.at_offset(q.offset() + returned)),
                cx.fresh,
            );
        }
        out.page = Some(PageInfo {
            offset: q.offset(),
            returned,
            total: page.total,
            pool: None,
            truncated: None,
        });
        out.body = (DiagnosticPage {
            items: page.items,
            count_unit: Some(DiagnosticCountUnit::RecallRecords),
            distinct_id_total: None,
        })
        .with_query(q.clone());
    }
    finish(&mut out, leg, error);
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
pub(super) fn get(cx: &Ctx<'_>, q: &DiagnosticGet, id: &biodata::DeviceRecallNumber) -> Outcome {
    let mut out = empty(biomcp_types::DiagnosticCard::into_reply(q.clone(), None));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&adapter::SPEC, deadline).fresh(cx.fresh);
    let mut error = None;
    match adapter::get(&leg, id) {
        Ok(Some(mut mapped)) => {
            crate::status::item(&mut out, true, false, false);
            out.include_refusals(mapped.refusals);
            out.problems.append(&mut mapped.problems);
            match adapter::classification(&leg, id) {
                Ok(mut classification) => {
                    mapped.value.classification = classification.value;
                    out.include_refusals(classification.refusals);
                    out.problems.append(&mut classification.problems);
                }
                Err(failure) => {
                    error = Some(failed(&mut out, q, failure));
                    crate::status::item(&mut out, true, false, true);
                }
            }
            if let Some(code) = &mapped.value.header.product_code
                && let Some(search) = crate::next::checked(
                    biomcp_types::AdverseEventSearch::maude_product(code.clone()),
                    Source::Maude,
                    &mut out.problems,
                )
            {
                next::push(
                    &mut out,
                    StepKind::Pivot,
                    &[biomcp_types::Source::Maude],
                    Request::SearchAdverseEvent(search),
                    cx.fresh,
                );
            }
            if !cx.fresh {
                next::push(
                    &mut out,
                    StepKind::Fresh,
                    &[q.source().source()],
                    Request::GetDiagnostic(q.clone()),
                    true,
                );
            }
            out.body = (DiagnosticCard::DeviceRecall(Box::new(mapped.value))).with_query(q.clone());
        }
        Ok(None) => crate::status::item(&mut out, false, false, false),
        Err(failure) => error = Some(failed(&mut out, q, failure)),
    }
    finish(&mut out, leg, error);
    command.finish(deadline.remaining());
    out
}
fn failed(out: &mut Outcome, q: &DiagnosticGet, failure: adapter::Failure) -> RequestError {
    if let Some(field) = failure.identity_field {
        out.problems.push(Problem {
            source: Source::FdaDeviceRecall,
            record: Some(q.record().clone()),
            field: field.into(),
            code: ProblemCode::IdentityConflict,
        });
    }
    failure.error
}
fn finish(out: &mut Outcome, leg: Leg<'_>, error: Option<RequestError>) {
    out.local = error.and_then(status::local);
    let source = status::source(Source::FdaDeviceRecall, leg.finish(), error);
    if out.status() == Status::Complete && (source.state == SourceState::Stale || out.dropped) {
        crate::status::item(out, true, false, true);
    }
    out.sources.push(source);
}
