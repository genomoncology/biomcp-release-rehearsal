//! Exact FAERS report reads and checked immutable section continuation.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod render;
use crate::{Ctx, Outcome, source_snapshot, status};
use biodata::{FaersReport, Text};
use biomcp_adapters::faers;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::{AdverseEventGet, ReplyData, Request, Source, SourceOutcome, StepKind};
#[derive(serde::Serialize, serde::Deserialize)]
struct Saved(
    FaersReport,
    Text,
    Vec<SourceOutcome>,
    Vec<biomcp_types::Problem>,
    #[serde(default)] faers::Positions,
    #[serde(default)] bool,
);
pub(super) fn get(cx: &Ctx<'_>, q: &AdverseEventGet) -> Outcome {
    let mut out = super::unsupported();
    out.body = biomcp_types::AdverseEventFaersCard::into_reply(q.clone(), None);
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    if let Err(e) = read(cx, q, deadline, &mut out) {
        out.local = status::local(e);
    }
    command.finish(deadline.remaining());
    out
}
fn key(q: &AdverseEventGet) -> Result<Vec<u8>, RequestError> {
    source_snapshot::key("faers", q.id())
}
fn read(
    cx: &Ctx<'_>,
    q: &AdverseEventGet,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&faers::SPEC, deadline).fresh(cx.fresh);
    // Source and component admission precede any retained payload read.
    let denied = if cx.fetch.disabled(Source::Faers) {
        Some(biomcp_fetch::SourceError::disabled(Source::Faers).into())
    } else {
        biomcp_adapters::adverse_event_sources::components::section(&leg, q.section()).err()
    };
    if let Some(e) = denied {
        out.sources
            .push(status::source(Source::Faers, leg.finish(), Some(e)));
        return Err(e);
    }
    let (saved, snapshot) = if let Some(snapshot) = q.snapshot() {
        let saved: Option<Saved> =
            source_snapshot::read(cx, &key(q)?, snapshot.as_str(), deadline)?;
        let Some(saved) = saved else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::run(
                StepKind::Restart,
                &[biomcp_types::Source::Faers],
                Request::GetAdverseEvent(q.restart()),
                false,
            ));
            return Ok(());
        };
        if saved.0.safetyreportid != *q.id() || !saved.4.check(&saved.0) {
            return Err(CacheError::Internal.into());
        }
        (saved, Some(snapshot.clone()))
    } else {
        let result = faers::detail::get(&leg, q.id());
        let error = result.as_ref().err().copied();
        out.sources
            .push(status::source(Source::Faers, leg.finish(), error));
        if matches!(error,Some(RequestError::Source(e)) if e.reason==biomcp_types::Reason::BadReply)
        {
            out.problems.push(biomcp_types::Problem {
                source: Source::Faers,
                record: Text::new(q.id().as_str()).ok(),
                field: "report_identity_or_shape".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
        }
        let detail = result?;
        let Some(report) = detail.report else {
            crate::status::item(out, false, false, false);
            return Ok(());
        };
        let saved = Saved(
            report,
            detail.report_url,
            out.sources.clone(),
            detail.problems,
            detail.positions,
            detail.refusals == biomcp_adapters::Refusal::Refused,
        );
        let snapshot = crate::publish::keep(
            source_snapshot::write(cx, &key(q)?, &saved, &out.sources, deadline),
            out,
        )?;
        (saved, snapshot)
    };
    render_saved(q, saved, snapshot, out)
}
fn render_saved(
    q: &AdverseEventGet,
    saved: Saved,
    snapshot: Option<Text>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    out.sources = saved.2.clone();
    out.dropped = saved.5;
    out.problems.extend(saved.3.clone());
    {
        let unavailable = out
            .sources
            .iter()
            .any(|s| s.state == biomcp_types::SourceState::Stale);
        crate::status::item(out, true, false, unavailable);
    };
    render::render(q, saved.0, saved.1, snapshot, saved.4, out)
}
