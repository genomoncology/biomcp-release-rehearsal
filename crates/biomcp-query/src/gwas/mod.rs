//! Source-local association search, without clinical interpretation.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod get;
pub(crate) mod map;
pub(crate) use get::get;
mod next;
mod studies;
mod threshold;
use crate::{Ctx, Outcome, status};
use biomcp_adapters::gwas;
use biomcp_types::{
    GwasPage, GwasPageQuery, GwasSearch, PageInfo, Problem, ProblemCode, ReplyData, Source,
    SourceState, Status,
};
/// One bounded native page; no per-hit metadata calls.
pub(crate) fn search(cx: &Ctx<'_>, q: &GwasSearch) -> Outcome {
    if q.studies() {
        return studies::search(cx, q);
    }
    if q.p_value().is_some() {
        return threshold::search(cx, q);
    }
    let mut out = empty(q);
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&gwas::SPEC, deadline).fresh(cx.fresh);
    let result = gwas::search(&leg, q);
    let error = result.as_ref().err().copied();
    let counts = result
        .as_ref()
        .ok()
        .map(|page| (page.rows.len() as u32, page.total));
    if let Ok(page) = result {
        out.include_refusals(page.refusals);
        out.problems.extend(page.problems);
        let items: Box<[_]> = page
            .rows
            .into_iter()
            .map(|row| map::association(row, &mut out.problems))
            .collect();
        let returned = items.len() as u32;
        let continuation =
            (u64::from(q.offset() - q.offset() % q.limit()) + u64::from(page.received)
                < page.total)
                .then(|| {
                    GwasPageQuery::Search(q.at_offset(
                        (q.offset() - q.offset() % q.limit()).saturating_add(page.received),
                    ))
                });
        let page = PageInfo {
            offset: q.offset(),
            returned,
            total: page.total,
            pool: None,
            truncated: None,
        };
        out.page = Some(page.clone());
        out.body = (GwasPage {
            items,
            page,
            continuation,
            qualifying_total: None,
        })
        .with_query(biomcp_types::GwasPageQuery::Search(q.clone()));
        {
            let unavailable = false;
            crate::status::item(&mut out, true, false, unavailable);
        };
    }
    let mut source = status::source(Source::Gwas, leg.finish(), error);
    if let Some((accepted, total)) = counts {
        source.retrieved_count = Some(accepted);
        source.count = Some(total);
    }
    if source.reason == Some(biomcp_types::Reason::BadReply) {
        out.problems.push(Problem {
            source: Source::Gwas,
            record: None,
            field: "association_page".into(),
            code: ProblemCode::ValueUnknown,
        });
    }
    if out.status() == Status::Complete && source.state == SourceState::Stale {
        crate::status::item(&mut out, true, false, true);
    }
    let ReplyData::GwasPage { body, .. } = &out.body else {
        return out;
    };
    out.next = next::search(q, body.as_ref(), source.state, cx.fresh, &mut out.problems);
    out.sources.push(source);
    status::search(&mut out);
    out.local = error.and_then(status::local);
    command.finish(deadline.remaining());
    out
}

fn empty(q: &GwasSearch) -> Outcome {
    crate::status::empty(biomcp_types::GwasPage::into_reply(
        biomcp_types::GwasPageQuery::Search(q.clone()),
        None,
    ))
}
