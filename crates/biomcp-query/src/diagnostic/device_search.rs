//! Source-owned FDA device search and document access.
use super::{empty, next};
use crate::{Ctx, Outcome, status};
use biomcp_types::{
    DiagnosticCountUnit, DiagnosticPage, DiagnosticSearch, PageInfo, Paged, ReplyBody, Request,
    Source, StepKind,
};
pub(super) fn search(cx: &Ctx<'_>, q: &DiagnosticSearch) -> Outcome {
    let source = q.source().source();
    let spec = if source == Source::FdaPma {
        &biomcp_adapters::fda_pma::SPEC
    } else {
        &biomcp_adapters::fda_510k::SPEC
    };
    let mut out = empty(
        (DiagnosticPage {
            items: Box::default(),
            count_unit: Some(DiagnosticCountUnit::DeviceSubmissions),
            distinct_id_total: None,
        })
        .with_query(q.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    let result = if q.source() == biomcp_types::DiagnosticSearchSource::FdaPma {
        biomcp_adapters::fda_pma::search(&leg, q)
    } else {
        biomcp_adapters::fda_510k::search(&leg, q)
    };
    let error = match result {
        Ok(page) => {
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
                count_unit: Some(DiagnosticCountUnit::DeviceSubmissions),
                distinct_id_total: None,
            })
            .with_query(q.clone());
            None
        }
        Err(error) => Some(error),
    };
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(source, leg.finish(), error));
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
