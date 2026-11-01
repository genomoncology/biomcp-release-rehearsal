//! The existing store retains one complete received ancestry collection.
use super::{
    Ctx, Deadline, GwasAncestries, GwasAncestry, GwasGet, Outcome, Problem, Reason, RequestError,
    Source, SourceOutcome, StepKind, action, gwas, page, rejected, status,
};
use crate::store::Store;
use biomcp_fetch::{CacheError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::ReplyData;
use sha2::{Digest, Sha256};
type Saved = (Vec<GwasAncestry>, Vec<SourceOutcome>, Vec<Problem>, bool);
pub(super) fn read(
    cx: &Ctx<'_>,
    q: &GwasGet,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let GwasGet::Study {
        id, page: window, ..
    } = q
    else {
        return Ok(());
    };
    let key = Sha256::digest(format!("biomcp.gwas-ancestries/6\0{id}"));
    let leg = cx.fetch.leg(&gwas::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::Gwas) {
        let error = RequestError::Source(SourceError {
            source: Source::Gwas,
            reason: Reason::Disabled,
            status: None,
            retry_after: None,
        });
        out.sources
            .push(status::source(Source::Gwas, leg.finish(), Some(error)));
        return Err(error);
    }
    let (saved, token) = if let Some(token) = window.snapshot() {
        let store = Store::open(cx.fetch.cache_access()?, deadline)?;
        let Some(payload) =
            store.source_snapshot(token.as_str(), &key, |s| cx.fetch.disabled(s))?
        else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::new(
                StepKind::Restart,
                &[biomcp_types::Source::Gwas],
                action(q.restart(), false),
            ));
            return Ok(());
        };
        let saved: Saved = serde_json::from_str(&payload).map_err(|_| CacheError::Snapshot)?;
        (saved, token.clone())
    } else {
        let rows = gwas::get::ancestries(&leg, id);
        out.sources.push(status::source(
            Source::Gwas,
            leg.finish(),
            rows.as_ref().err().copied(),
        ));
        let rows = rows?;
        out.include_refusals(rows.refusals);
        out.problems.extend(rows.problems);
        rejected(out, rows.rejected);
        let saved = (
            rows.items,
            out.sources.clone(),
            out.problems.clone(),
            out.dropped,
        );
        let payload = serde_json::to_string(&saved).map_err(|_| CacheError::Snapshot)?;
        let token = Store::open(cx.fetch.cache_access()?, deadline)?.save_source_snapshot(
            &key,
            &payload,
            &out.sources,
        )?;
        (saved, token)
    };
    out.sources = saved.1;
    out.problems = saved.2;
    out.dropped = saved.3;
    {
        let unavailable = false;
        crate::status::item(out, true, false, unavailable);
    };
    let navigation = q.with_page(window.with_snapshot(token));
    let (items, continuation) = page(&navigation, &saved.0, out);
    out.body = (GwasAncestries {
        study: id.clone(),
        items,
        continuation,
    })
    .with_query(q.clone());
    Ok(())
}
