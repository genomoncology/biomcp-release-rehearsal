//! Explicit diagnostic source ownership; reads consume only complete local snapshots.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod clearance;
mod companion;
mod device_search;
mod get;
mod next;
mod pma;
mod recall;
mod summary;
mod who;
pub(super) use crate::status::empty;
use crate::{Action, Bundle, Ctx, Outcome, status};
use biomcp_adapters::gtr;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{
    DiagnosticPage, DiagnosticSearch, DiagnosticSearchSource, PageInfo, Request, Source,
    SourceState, Status, StepKind,
};
pub(crate) use get::get;
fn finish(out: &mut Outcome, leg: Leg<'_>, error: Option<RequestError>) {
    finish_bundle(out, leg, error, Source::Gtr, Bundle::GTR);
}
fn finish_bundle(
    out: &mut Outcome,
    leg: Leg<'_>,
    error: Option<RequestError>,
    source: Source,
    bundle: Bundle,
) {
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(source, leg.finish(), error));
    if error.is_some() {
        crate::status::item(out, false, true, false);
    } else if (out.dropped || out.sources.iter().any(|s| s.state == SourceState::Stale))
        && out.status() == Status::Complete
    {
        crate::status::item(out, true, false, true);
    }
    if out
        .sources
        .iter()
        .any(|s| matches!(s.state, SourceState::Syncing | SourceState::Stale))
        && !out
            .next
            .iter()
            .any(|step| step.1 == Action::Sync { bundle })
    {
        out.next.push(crate::next::Step::new(
            StepKind::Sync,
            &[source],
            Action::Sync { bundle },
        ));
    }
}
/// Search only stored source-native GTR relations and retain matched selectors.
pub(crate) fn search(cx: &Ctx<'_>, q: &DiagnosticSearch) -> Outcome {
    if matches!(
        q.source(),
        DiagnosticSearchSource::FdaPma | DiagnosticSearchSource::Fda510k
    ) {
        return device_search::search(cx, q);
    }
    if q.source() == DiagnosticSearchSource::FdaDeviceRecall {
        return recall::search(cx, q);
    }
    if q.source() == DiagnosticSearchSource::FdaCdx {
        return companion::search(cx, q);
    }
    let (spec, source, bundle, count) = search_source(q.source());
    let who = q.source() == DiagnosticSearchSource::WhoIvd;
    let mut out = empty(
        (DiagnosticPage {
            items: Box::default(),
            count_unit: Some(count),
            distinct_id_total: None,
        })
        .with_query(q.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(spec, deadline);
    let result = if who {
        biomcp_adapters::who_ivd::search(&leg, q).map(|p| (p.items, p.total, p.info))
    } else {
        gtr::search(&leg, q).map(|p| {
            out.include_refusals(p.refusals);
            status::append(&mut out.problems, p.problems);
            (p.items, p.total, p.info)
        })
    };
    let error = result.as_ref().err().copied();
    if let Ok((items, total, info)) = result {
        crate::status::item(&mut out, true, false, false);
        let bound = info
            .content_revision
            .as_deref()
            .and_then(|v| v.parse().ok())
            .map_or_else(|| q.clone(), |v| q.with_bundle_revision(v));
        next::candidates(&mut out, &bound, &items, cx.fresh);
        let returned = items.len() as u32;
        if u64::from(bound.offset()) + u64::from(returned) < total {
            next::push(
                &mut out,
                StepKind::Page,
                &[bound.source().source()],
                Request::SearchDiagnostic(bound.at_offset(bound.offset() + returned)),
                cx.fresh,
            );
        }
        out.page = Some(PageInfo {
            offset: bound.offset(),
            returned,
            total,
            pool: None,
            truncated: None,
        });
        out.body = (DiagnosticPage {
            items,
            count_unit: Some(count),
            distinct_id_total: None,
        })
        .with_query(q.clone());
    }
    finish_bundle(&mut out, leg, error, source, bundle);
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
fn search_source(
    source: DiagnosticSearchSource,
) -> (
    &'static biomcp_fetch::SourceSpec,
    Source,
    Bundle,
    biomcp_types::DiagnosticCountUnit,
) {
    if source == DiagnosticSearchSource::WhoIvd {
        (
            &biomcp_adapters::who_ivd::SPEC,
            Source::WhoIvd,
            Bundle::WHO_IVD,
            biomcp_types::DiagnosticCountUnit::ProductIdentities,
        )
    } else {
        (
            &gtr::SPEC,
            Source::Gtr,
            Bundle::GTR,
            biomcp_types::DiagnosticCountUnit::TestIdentities,
        )
    }
}
fn unsupported(source: Source, mut out: Outcome) -> Outcome {
    out.sources.push(biomcp_adapters::registry::outcome(
        source,
        SourceState::Skipped,
        Some(biomcp_types::Reason::Unsupported),
    ));
    out
}
