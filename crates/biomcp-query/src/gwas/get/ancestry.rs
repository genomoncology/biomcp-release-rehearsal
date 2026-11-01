//! The existing store retains one complete received ancestry collection.
use super::{
    Ctx, Deadline, GwasAncestries, GwasAncestry, GwasGet, Outcome, Problem, Reason, RequestError,
    Source, SourceOutcome, StepKind, action, gwas, page, rejected, status,
};
use crate::source_snapshot;
use biomcp_fetch::SourceError;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::ReplyData;
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
    let key = source_snapshot::key("gwas-ancestry", id)?;
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
        let saved: Option<Saved> = source_snapshot::read(cx, &key, token.as_str(), deadline)?;
        let Some(saved) = saved else {
            out.body = ReplyData::PoolError { mismatch: false };
            out.next.push(crate::next::Step::new(
                StepKind::Restart,
                &[biomcp_types::Source::Gwas],
                action(q.restart(), false),
            ));
            return Ok(());
        };
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
        let token = source_snapshot::write(cx, &key, &saved, &out.sources, deadline)?;
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
