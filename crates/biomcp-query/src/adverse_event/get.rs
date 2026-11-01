//! Exact FAERS report reads and checked immutable section continuation.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod render;
use crate::{Ctx, Outcome, status, store::Store};
use biodata::{FaersReport, Text};
use biomcp_adapters::faers;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::{
    AdverseEventGet, AdverseEventSection, ReplyData, Request, Source, SourceOutcome, StepKind,
};
use sha2::{Digest, Sha256};
#[derive(serde::Deserialize)]
struct Saved(
    FaersReport,
    Text,
    Vec<SourceOutcome>,
    Vec<biomcp_types::Problem>,
    #[serde(default)] faers::Positions,
    #[serde(default)] bool,
);
impl serde::Serialize for Saved {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTuple;
        let mut tuple = serializer.serialize_tuple(if self.5 {
            6
        } else if self.4.is_empty() {
            4
        } else {
            5
        })?;
        tuple.serialize_element(&self.0)?;
        tuple.serialize_element(&self.1)?;
        tuple.serialize_element(&self.2)?;
        tuple.serialize_element(&self.3)?;
        if !self.4.is_empty() || self.5 {
            tuple.serialize_element(&self.4)?;
        }
        if self.5 {
            tuple.serialize_element(&true)?;
        }
        tuple.end()
    }
}
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
fn key(q: &AdverseEventGet) -> Vec<u8> {
    Sha256::digest(format!("biomcp.faers_snapshot/2\0{}", q.id()).as_bytes()).to_vec()
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
        Some(
            biomcp_fetch::SourceError {
                source: Source::Faers,
                reason: biomcp_types::Reason::Disabled,
                status: None,
                retry_after: None,
            }
            .into(),
        )
    } else if q.section() == Some(AdverseEventSection::Reactions) {
        biomcp_adapters::adverse_event_sources::components::meddra(&leg).err()
    } else {
        None
    };
    if let Some(e) = denied {
        out.sources
            .push(status::source(Source::Faers, leg.finish(), Some(e)));
        return Err(e);
    }
    let (saved, snapshot) = if let Some(snapshot) = q.snapshot() {
        let store = Store::open(cx.fetch.cache_access()?, deadline)?;
        let Some(payload) =
            store.source_snapshot(snapshot.as_str(), &key(q), |s| cx.fetch.disabled(s))?
        else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::run(
                StepKind::Restart,
                &[biomcp_types::Source::Faers],
                Request::GetAdverseEvent(q.restart()),
                false,
            ));
            return Ok(());
        };
        let saved: Saved = serde_json::from_str(&payload).map_err(|_| CacheError::Snapshot)?;
        if saved.0.safetyreportid != *q.id() || !saved.4.check(&saved.0) {
            return Err(CacheError::Database.into());
        }
        (saved, snapshot.clone())
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
        let payload = serde_json::to_string(&saved).map_err(|_| CacheError::Snapshot)?;
        let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
        let snapshot = store.save_source_snapshot(&key(q), &payload, &out.sources)?;
        (saved, snapshot)
    };
    render_saved(q, saved, snapshot, out)
}
fn render_saved(
    q: &AdverseEventGet,
    saved: Saved,
    snapshot: Text,
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
