use crate::{Ctx, Outcome, status, store::Store};
use biodata::{OrcidId, OrcidWorkGroup};
use biomcp_adapters::orcid;
use biomcp_fetch::{CacheError, Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    AuthorPapersQuery, OrcidWorks, OrcidWorksScope, PageInfo, Reason, ReplyData, Request, Source,
    SourceOutcome, SourceState, StepKind,
};
/// Read one whole source reply and retain local group pages.
pub(super) fn papers(cx: &Ctx<'_>, q: &AuthorPapersQuery, id: &OrcidId) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = super::empty(biomcp_types::OrcidWorks::into_reply(q.clone(), None));
    if let Err(error) = execute(cx, q, id, deadline, &mut out) {
        out.local = status::local(error);
        if out.sources.is_empty() {
            out.sources.push(biomcp_adapters::registry::outcome(
                Source::Orcid,
                SourceState::Failed,
                None,
            ));
        }
        if matches!(error, RequestError::Source(e) if e.reason == Reason::NotFound) {
            crate::status::item(&mut out, false, false, false);
        }
    }
    super::retry(&mut out, Request::AuthorPapers(q.restart()));
    command.finish(deadline.remaining());
    out
}
fn restart(q: &AuthorPapersQuery, out: &mut Outcome, mismatch: bool) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::new(
        StepKind::Restart,
        &[biomcp_types::Source::Orcid],
        super::run(Request::AuthorPapers(q.restart())),
    ));
}
fn key(q: &AuthorPapersQuery) -> Result<Vec<u8>, RequestError> {
    serde_json::to_vec(&(
        "biomcp.orcid_works_snapshot/4",
        q.id(),
        q.section(),
        q.limit(),
    ))
    .map_err(|_| CacheError::Database.into())
}
fn execute(
    cx: &Ctx<'_>,
    q: &AuthorPapersQuery,
    id: &OrcidId,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if cx.fetch.disabled(Source::Orcid) {
        out.sources.push(biomcp_adapters::registry::outcome(
            Source::Orcid,
            SourceState::Skipped,
            Some(Reason::Disabled),
        ));
        return Err(SourceError {
            source: Source::Orcid,
            reason: Reason::Disabled,
            status: None,
            retry_after: None,
        }
        .into());
    }
    let expected = key(q)?;
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let saved = if cx.fresh {
        None
    } else {
        store.find(&expected, q.pool().map(biodata::Text::as_str))?
    };
    let mismatch = saved.as_ref().is_some_and(|p| p.key != expected);
    let (groups, source, pool) = if let Some(pool) = saved.filter(|_| !mismatch) {
        let payload = store.source_snapshot(pool.info.id.as_str(), &expected, |source| {
            cx.fetch.disabled(source)
        })?;
        let Some(payload) = payload else {
            restart(q, out, false);
            return Ok(());
        };
        let (groups, source, problems, dropped): (
            Box<[OrcidWorkGroup]>,
            SourceOutcome,
            Vec<biomcp_types::Problem>,
            bool,
        ) = serde_json::from_str(&payload).map_err(|_| CacheError::Snapshot)?;
        out.problems.extend(problems);
        out.dropped = dropped;
        (groups, source, pool)
    } else {
        if q.pool().is_some() {
            restart(q, out, mismatch);
            return Ok(());
        }
        let leg = cx.fetch.leg(&orcid::SPEC, deadline).fresh(cx.fresh);
        let reply = orcid::works(&leg, id);
        let source = status::source(
            Source::Orcid,
            leg.finish(),
            super::source_error(reply.as_ref().err().copied()),
        );
        out.sources.push(source.clone());
        let mapped = reply?;
        let groups = mapped.value;
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        let payload = serde_json::to_string(&(&groups, &source, &out.problems, out.dropped))
            .map_err(|_| CacheError::Snapshot)?;
        let size = u32::try_from(groups.len()).map_err(|_| CacheError::Database)?;
        let pool =
            store.publish_source_pool(&expected, &payload, size, std::slice::from_ref(&source))?;
        (groups, source, pool)
    };
    present(q, id, out, groups, source, pool);
    Ok(())
}
fn present(
    q: &AuthorPapersQuery,
    id: &OrcidId,
    out: &mut Outcome,
    groups: Box<[OrcidWorkGroup]>,
    source: SourceOutcome,
    pool: crate::store::Pool,
) {
    let total = groups.len() as u64;
    let rows: Box<[_]> = groups
        .into_iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .collect();
    let returned = rows.len() as u32;
    let offset = q.offset().saturating_add(returned);
    if u64::from(offset) < total && returned > 0 {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::Orcid],
            super::run(Request::AuthorPapers(
                q.at_offset(offset).with_pool(pool.info.id.clone()),
            )),
        ));
    }
    pivots(out, &rows);
    out.page = Some(PageInfo {
        offset: q.offset(),
        returned,
        total,
        pool: Some(pool.info),
        truncated: None,
    });
    {
        let unavailable = source.state == SourceState::Stale;
        crate::status::item(out, true, false, unavailable);
    };
    out.sources = vec![source];
    out.body = (OrcidWorks {
        collection_scope: OrcidWorksScope::WholePublicWorksReply,
        author: id.clone(),
        groups: rows,
        observed_groups: total,
    })
    .with_query(q.clone());
}
fn pivots(out: &mut Outcome, rows: &[OrcidWorkGroup]) {
    for id in rows
        .iter()
        .flat_map(|g| &g.assertions)
        .flat_map(|a| &a.external_ids)
    {
        if id.relationship.as_ref().map(biodata::Text::as_str) != Some("self") {
            continue;
        }
        let input = match id.namespace.as_deref() {
            Some("pmid") => crate::next::checked(
                id.id.as_str().parse(),
                biomcp_types::Source::Orcid,
                &mut out.problems,
            )
            .map(biomcp_types::ArticleInput::Pmid),
            Some("doi") => crate::next::checked(
                id.id.as_str().parse(),
                biomcp_types::Source::Orcid,
                &mut out.problems,
            )
            .map(biomcp_types::ArticleInput::Doi),
            _ => None,
        };
        if let Some(input) = input
            && let Some(q) = crate::next::checked(
                biomcp_types::ArticleGet::new(vec![input].into()),
                biomcp_types::Source::Orcid,
                &mut out.problems,
            )
        {
            let step = crate::next::Step::new(
                StepKind::Pivot,
                &super::super::article::next::get_sources(&q),
                super::run(Request::GetArticle(q)),
            );
            if out.next.len() < 5 && !out.next.contains(&step) {
                out.next.push(step);
            }
        }
    }
}
