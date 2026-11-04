use crate::{Action, Bundle, Ctx, Outcome, status};
use biodata::CompanionDiagnosticRelation;
use biomcp_adapters::fda_cdx;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiagnosticCompanion, DiagnosticSearch, DrugGet, DrugSource, PageInfo, Reason, Request, Source,
    SourceState, Status, StepKind,
};
use std::collections::HashSet;

pub(super) fn search(cx: &Ctx<'_>, q: &DiagnosticSearch) -> Outcome {
    let echo = q;
    let mut out = super::empty(
        (DiagnosticCompanion {
            items: Box::default(),
            count_unit: None,
            distinct_id_total: None,
        })
        .with_query(q.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&fda_cdx::SPEC, deadline);
    let result = fda_cdx::search(&leg, q);
    let error = result.as_ref().err().copied();
    let mut group_unsupported = false;
    if let Ok(page) = result {
        let q = page
            .info
            .content_revision
            .as_deref()
            .and_then(|v| v.parse().ok())
            .map_or_else(|| q.clone(), |v| q.with_bundle_revision(v));
        pivots(&mut out, &page.items, cx.fresh);
        let returned = page.items.len() as u32;
        if u64::from(q.offset()) + u64::from(returned) < page.total {
            super::next::push(
                &mut out,
                StepKind::Page,
                &[q.source().source()],
                Request::SearchDiagnostic(q.at_offset(q.offset() + returned)),
                cx.fresh,
            );
        }
        {
            let unavailable = false;
            crate::status::item(&mut out, true, false, unavailable);
        };
        group_unsupported = page.group_unsupported;
        out.problems.extend(page.problems);
        out.page = Some(PageInfo {
            offset: q.offset(),
            returned,
            total: page.total,
            pool: None,
            truncated: None,
        });
        out.body = (DiagnosticCompanion {
            items: page.items,
            count_unit: None,
            distinct_id_total: None,
        })
        .with_query(echo.clone());
    }
    out.local = error.and_then(status::local);
    let mut source = status::source(Source::FdaCdx, leg.finish(), error);
    if group_unsupported {
        source.reason = Some(Reason::Unsupported);
    }
    if matches!(source.state, SourceState::Stale) && out.status() == Status::Complete {
        crate::status::item(&mut out, true, false, true);
    }
    if matches!(source.state, SourceState::Syncing | SourceState::Stale) {
        out.next.push(crate::next::Step::new(
            StepKind::Sync,
            &[biomcp_types::Source::FdaCdx],
            Action::Sync {
                bundle: Bundle::FDA_CDX,
            },
        ));
    }
    out.sources.push(source);
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
fn pivots(out: &mut Outcome, rows: &[CompanionDiagnosticRelation], fresh: bool) {
    let mut seen = HashSet::new();
    for row in rows {
        let (devices, applications) = match row {
            CompanionDiagnosticRelation::Individual {
                device_ids,
                drug_applications,
                ..
            }
            | CompanionDiagnosticRelation::Group {
                device_ids,
                drug_applications,
                ..
            } => (device_ids, drug_applications),
        };
        for device in devices {
            if !(device.as_str().starts_with('P') || device.as_str().starts_with('K'))
                || !seen.insert(device.as_str().to_owned())
            {
                continue;
            }
            let mut parts = device.as_str().split('/');
            if let Some(base) = parts.next() {
                let source = if base.starts_with('K') {
                    biomcp_types::DiagnosticGetSource::Fda510k
                } else {
                    biomcp_types::DiagnosticGetSource::FdaPma
                };
                let get =
                    base.parse()
                        .map_err(|_| biomcp_types::QueryInvalid::new(&["id"]))
                        .and_then(|id| {
                            let supplement =
                                parts.next().map(str::parse).transpose().map_err(|_| {
                                    biomcp_types::QueryInvalid::new(&["supplement"])
                                })?;
                            biomcp_types::DiagnosticGet::new(id, source, supplement)
                        });
                if let Some(get) = crate::next::checked(get, source.source(), &mut out.problems) {
                    super::next::push(
                        out,
                        StepKind::Pivot,
                        &[get.source().source()],
                        Request::GetDiagnostic(get),
                        fresh,
                    );
                }
            }
        }
        for application in applications {
            if seen.insert(application.id.as_str().to_owned())
                && let Some(id) = crate::next::checked(
                    application.id.as_str().parse(),
                    biomcp_types::Source::FdaCdx,
                    &mut out.problems,
                )
                && let Some(get) = crate::next::checked(
                    DrugGet::new(id, DrugSource::Drugsfda),
                    biomcp_types::Source::FdaCdx,
                    &mut out.problems,
                )
            {
                super::next::push(
                    out,
                    StepKind::Pivot,
                    &[get.source().source()],
                    Request::GetDrug(get),
                    fresh,
                );
            }
        }
    }
}
