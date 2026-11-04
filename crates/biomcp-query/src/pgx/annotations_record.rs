//! Retained complete ClinPGx collections reuse the shared immutable source store.
use crate::{Ctx, Outcome, source_snapshot, status};
use biomcp_adapters::clinpgx;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::{
    PgxAnnotationKind, PgxAnnotations, PgxGet, ReplyData, Request, Source, SourceOutcome,
    SourceState, StepKind,
};
type Saved = (
    PgxAnnotations,
    Vec<SourceOutcome>,
    Vec<biomcp_types::Problem>,
    bool,
);
fn key(q: &PgxGet) -> Result<Vec<u8>, RequestError> {
    source_snapshot::key("pgx", &(q.gene(), q.kind()))
}
pub(super) fn load(
    cx: &Ctx<'_>,
    q: &PgxGet,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(PgxAnnotations, Option<biodata::Text>)>, RequestError> {
    let leg = cx.fetch.leg(&clinpgx::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::ClinPgx) {
        let error = RequestError::Source(SourceError::disabled(Source::ClinPgx));
        out.sources
            .push(status::source(Source::ClinPgx, leg.finish(), Some(error)));
        return Err(error);
    }
    let (saved, snapshot) = if let Some(snapshot) = q.snapshot() {
        let saved: Option<Saved> =
            source_snapshot::read(cx, &key(q)?, snapshot.as_str(), deadline)?;
        let Some(saved) = saved else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::run(
                StepKind::Restart,
                &[q.source().source()],
                Request::GetPgx(q.restart()),
                false,
            ));
            return Ok(None);
        };
        (saved, Some(snapshot.clone()))
    } else {
        let reply = match q.kind() {
            Some(PgxAnnotationKind::Clinical) => clinpgx::clinical(&leg, q).map(|rows| {
                out.include_refusals(rows.refusals);
                out.problems.extend(rows.problems);
                super::annotations::body(q, rows.value, vec![])
            }),
            _ => clinpgx::references(&leg, q).map(|rows| {
                out.include_refusals(rows.refusals);
                out.problems.extend(rows.problems);
                super::annotations::body(q, vec![], rows.value)
            }),
        };
        let error = reply.as_ref().err().copied();
        out.sources
            .push(status::source(Source::ClinPgx, leg.finish(), error));
        out.problems
            .extend(out.sources.iter().filter_map(super::record_problem));
        let saved = (
            reply?,
            out.sources.clone(),
            out.problems.clone(),
            out.dropped,
        );
        let snapshot = crate::publish::keep(
            source_snapshot::write(cx, &key(q)?, &saved, &out.sources, deadline),
            out,
        )?;
        (saved, snapshot)
    };
    out.sources = saved.1;
    out.problems = saved.2;
    out.dropped = saved.3;
    {
        let (found, identity_failed, unavailable) =
            if out.sources.iter().any(|s| s.state == SourceState::Stale) {
                (true, false, true)
            } else {
                (true, false, false)
            };
        crate::status::item(out, found, identity_failed, unavailable);
    };
    Ok(Some((saved.0, snapshot)))
}
