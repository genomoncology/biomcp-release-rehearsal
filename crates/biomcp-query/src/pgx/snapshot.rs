//! Retained complete ClinPGx collections reuse the shared immutable source store.
use crate::{Ctx, Outcome, status, store::Store};
use biomcp_adapters::clinpgx;
use biomcp_fetch::{CacheError, Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::{
    PgxAnnotationKind, PgxAnnotations, PgxGet, Reason, ReplyData, Request, Source, SourceOutcome,
    SourceState, StepKind,
};
use sha2::{Digest, Sha256};
type Saved = (
    PgxAnnotations,
    Vec<SourceOutcome>,
    Vec<biomcp_types::Problem>,
    bool,
);
fn key(q: &PgxGet) -> Vec<u8> {
    Sha256::digest(
        format!(
            "biomcp.clinpgx_annotations/4\0{}\0{}",
            q.gene(),
            q.kind()
                .map(PgxAnnotationKind::as_str)
                .unwrap_or("clinical")
        )
        .as_bytes(),
    )
    .to_vec()
}
pub(super) fn load(
    cx: &Ctx<'_>,
    q: &PgxGet,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(PgxAnnotations, biodata::Text)>, RequestError> {
    let leg = cx.fetch.leg(&clinpgx::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::ClinPgx) {
        let error = RequestError::Source(SourceError {
            source: Source::ClinPgx,
            reason: Reason::Disabled,
            status: None,
            retry_after: None,
        });
        out.sources
            .push(status::source(Source::ClinPgx, leg.finish(), Some(error)));
        return Err(error);
    }
    let (saved, snapshot) = if let Some(snapshot) = q.snapshot() {
        let store = Store::open(cx.fetch.cache_access()?, deadline)?;
        let Some(payload) = store.source_snapshot(snapshot.as_str(), &key(q), |source| {
            cx.fetch.disabled(source)
        })?
        else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::run(
                StepKind::Restart,
                &[q.source().source()],
                Request::GetPgx(q.restart()),
                false,
            ));
            return Ok(None);
        };
        let saved: Saved = serde_json::from_str(&payload).map_err(|_| CacheError::Snapshot)?;
        (saved, snapshot.clone())
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
        let payload = serde_json::to_string(&saved).map_err(|_| CacheError::Snapshot)?;
        let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
        let snapshot = store.save_source_snapshot(&key(q), &payload, &out.sources)?;
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
