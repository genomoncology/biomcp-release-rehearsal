//! Source-native drug recall search and exact retrieval.
use super::empty;
use crate::{Ctx, Outcome, status};
use biomcp_adapters::fda_drug_recall as source;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugGet, DrugRecallPage, DrugSearch, DrugSource, PageInfo, Request, Source, SourceState,
    Status, StepKind,
};
pub(super) fn search(cx: &Ctx<'_>, q: &DrugSearch) -> Outcome {
    let mut out = empty(
        (DrugRecallPage {
            count_unit: biomcp_types::DrugRecallCountUnit::RecallRecords,
            items: Box::default(),
            total: 0,
        })
        .with_query(q.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&source::SPEC, deadline).fresh(cx.fresh);
    let result = source::search(&leg, q);
    let error = result.as_ref().err().copied();
    if let Ok(page) = result {
        let returned = page.items.len() as u32;
        {
            let found = returned != 0;
            crate::status::item(&mut out, found, false, false);
        };
        out.problems = page.problems;
        for item in &page.items {
            if let Some(id) = crate::next::checked(
                item.recall_number.to_string().parse(),
                biomcp_types::Source::FdaDrugRecall,
                &mut out.problems,
            ) && let Some(get) = crate::next::checked(
                DrugGet::new(id, DrugSource::FdaDrugRecall),
                biomcp_types::Source::FdaDrugRecall,
                &mut out.problems,
            ) {
                out.next.push(crate::next::Step::run(
                    StepKind::Disambiguate,
                    &[biomcp_types::Source::FdaDrugRecall],
                    Request::GetDrug(get),
                    cx.fresh,
                ));
            }
        }
        if let Some(offset) = q
            .offset()
            .checked_add(returned)
            .filter(|n| returned > 0 && u64::from(*n) < page.total)
        {
            out.next.push(crate::next::Step::run(
                StepKind::Page,
                &[biomcp_types::Source::FdaDrugRecall],
                Request::SearchDrug(q.at_offset(offset)),
                cx.fresh,
            ));
        }
        out.page = Some(PageInfo {
            offset: q.offset(),
            returned,
            total: page.total,
            pool: None,
            truncated: None,
        });
        out.body = (DrugRecallPage {
            count_unit: biomcp_types::DrugRecallCountUnit::RecallRecords,
            items: page.items,
            total: page.total,
        })
        .with_query(q.clone());
    }
    finish(&mut out, leg, error);
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
pub(super) fn get(cx: &Ctx<'_>, q: &DrugGet) -> Outcome {
    let mut out = empty(biomcp_types::DrugRecallCard::into_reply(q.clone(), None));
    let Some(id) = q.recall_number() else {
        return out;
    };
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&source::SPEC, deadline).fresh(cx.fresh);
    let mut error = None;
    match source::get(&leg, id) {
        Ok(Some(row)) => {
            crate::status::item(&mut out, true, false, false);
            out.include_refusals(row.refusals);
            out.problems = row.problems;
            out.body = (row.value).with_query(q.clone());
        }
        Ok(None) => crate::status::item(&mut out, false, false, false),
        Err(failure) => {
            error = Some(failure.error);
            if let Some(field) = failure.identity_field {
                out.problems.push(biomcp_types::Problem {
                    source: Source::FdaDrugRecall,
                    record: q.input().to_string().parse().ok(),
                    field: field.into(),
                    code: biomcp_types::ProblemCode::IdentityConflict,
                });
            }
        }
    }
    finish(&mut out, leg, error);
    command.finish(deadline.remaining());
    out
}
fn finish(out: &mut Outcome, leg: Leg<'_>, error: Option<RequestError>) {
    out.local = error.and_then(status::local);
    let source = status::source(Source::FdaDrugRecall, leg.finish(), error);
    if out.status() == Status::Complete && (source.state == SourceState::Stale || out.dropped) {
        crate::status::item(out, true, false, true);
    }
    out.sources.push(source);
}
