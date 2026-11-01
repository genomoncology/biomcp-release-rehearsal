//! Reuse the shared source snapshot store; admission precedes every retained read.
use super::{
    AdverseEventGet, AdverseEventSection, Ctx, Outcome, Request, Source, StepKind, Text, maude,
};
use crate::{source_snapshot, status};
use biomcp_fetch::{CacheError, Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyData;
use biomcp_types::{Reason, SourceOutcome};
#[derive(serde::Serialize, serde::Deserialize)]
struct Saved(
    biodata::MaudeReport,
    Vec<SourceOutcome>,
    #[serde(default)] Vec<biomcp_types::Problem>,
    #[serde(default)] maude::Positions,
    #[serde(default)] bool,
);
fn key(query: &AdverseEventGet) -> Result<Vec<u8>, RequestError> {
    source_snapshot::key("adverse_event_maude", query.id())
}
pub(super) fn load(
    cx: &Ctx<'_>,
    query: &AdverseEventGet,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(biodata::MaudeReport, maude::Positions, Option<Text>)>, RequestError> {
    let leg = cx.fetch.leg(&maude::SPEC, deadline).fresh(cx.fresh);
    let denied = if cx.fetch.disabled(Source::Maude) {
        Some(SourceError::disabled(Source::Maude).into())
    } else if query.section() == Some(AdverseEventSection::Devices) {
        biomcp_adapters::adverse_event_sources::components::gmdn(&leg).err()
    } else {
        None
    };
    if let Some(error) = denied {
        out.sources
            .push(status::source(Source::Maude, leg.finish(), Some(error)));
        return Err(error);
    }
    let (saved, snapshot) = if let Some(snapshot) = query.snapshot() {
        let saved: Option<Saved> =
            source_snapshot::read(cx, &key(query)?, snapshot.as_str(), deadline)?;
        let Some(saved) = saved else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::run(
                StepKind::Restart,
                &[biomcp_types::Source::Maude],
                Request::GetAdverseEvent(query.restart()),
                false,
            ));
            return Ok(None);
        };
        if saved.0.mdr_report_key != *query.id() || !saved.3.check(&saved.0) {
            return Err(CacheError::Database.into());
        }
        (saved, Some(snapshot.clone()))
    } else {
        let reply = maude::get(&leg, query);
        let error = reply.as_ref().err().copied();
        out.sources
            .push(status::source(Source::Maude, leg.finish(), error));
        if matches!(error,Some(RequestError::Source(e)) if e.reason==Reason::BadReply) {
            out.problems.push(biomcp_types::Problem {
                source: Source::Maude,
                record: Text::new(query.id().as_str()).ok(),
                field: "report".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
        }
        let Some(report) = reply? else {
            crate::status::item(out, false, false, false);
            return Ok(None);
        };
        let saved = Saved(
            report.value.data,
            out.sources.clone(),
            report.problems,
            report.value.positions,
            report.refusals == biomcp_adapters::Refusal::Refused,
        );
        let snapshot = crate::publish::keep(
            source_snapshot::write(cx, &key(query)?, &saved, &out.sources, deadline),
            out,
        )?;
        (saved, snapshot)
    };
    Ok(Some(restore(saved, snapshot, out)))
}
fn restore(
    saved: Saved,
    snapshot: Option<Text>,
    out: &mut Outcome,
) -> (biodata::MaudeReport, maude::Positions, Option<Text>) {
    out.sources = saved.1;
    out.dropped = saved.4;
    out.problems.extend(saved.2);
    crate::status::item(out, true, false, false);
    (saved.0, saved.3, snapshot)
}
