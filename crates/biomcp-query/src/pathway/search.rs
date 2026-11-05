use super::{empty, run};
use crate::{Ctx, Outcome, source_snapshot, status};
use biomcp_adapters::reactome;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PathwayGet, PathwayPage, PathwayPageInfo, PathwaySearch, PathwayTotalKind, Reason, ReplyData,
    Request, Source, StepKind,
};
fn page(q: &PathwaySearch) -> PathwayPage {
    PathwayPage {
        items: Box::default(),
        page: PathwayPageInfo {
            offset: q.offset(),
            returned: 0,
            total: 0,
            total_kind: PathwayTotalKind::Source,
            next_offset: None,
            pool: None,
        },
    }
}
/// Read a retained native page or request only its next bounded native window.
pub(crate) fn search(cx: &Ctx<'_>, q: &PathwaySearch) -> Outcome {
    if q.sources().len() > 1 {
        return super::union::search(cx, q);
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = empty((page(q)).with_query(q.clone()));
    let source = if q.source() == biomcp_types::PathwaySource::Wikipathways {
        Source::WikiPathways
    } else {
        Source::Reactome
    };
    if let Err(error) = execute(cx, q, deadline, &mut out) {
        if out.sources.iter().all(|s| s.source != source) {
            let reason = crate::status::reason(error);
            out.sources.push(biomcp_adapters::registry::outcome(
                source,
                if reason == Some(Reason::Disabled) {
                    biomcp_types::SourceStatus::Skipped(reason)
                } else {
                    biomcp_types::SourceStatus::Failed(reason)
                },
            ));
        }
        out.local = status::local(error);
        crate::status::item(&mut out, false, true, false);
    }
    super::wikipathways::sync_step(&mut out);
    command.finish(deadline.remaining());
    out
}
fn execute(
    cx: &Ctx<'_>,
    q: &PathwaySearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let source = if q.source() == biomcp_types::PathwaySource::Wikipathways {
        Source::WikiPathways
    } else {
        Source::Reactome
    };
    if cx.fetch.disabled(source) {
        return Err(SourceError::disabled(source).into());
    }
    let key = source_snapshot::key("pathway-search", &q.restart())?;
    let saved: Option<Saved> = if let Some(id) = q.pool().filter(|_| !(cx.fresh || q.fresh())) {
        let saved: Option<Saved> = source_snapshot::read(cx, &key, id.as_str(), deadline)?;
        match saved {
            Some(saved) if saved.complete || q.offset() == saved.body.page.offset => Some(saved),
            Some(saved) if saved.body.page.next_offset == Some(q.offset()) => None,
            _ => {
                out.body = ReplyData::PoolError { mismatch: false };
                out.next.push(crate::next::Step::new(
                    StepKind::Restart,
                    &super::next::search(q),
                    run(Request::SearchPathway(q.restart())),
                ));
                None
            }
        }
    } else {
        None
    };
    if matches!(out.body, ReplyData::PoolError { .. }) {
        return Ok(());
    }
    let saved = match saved {
        Some(saved) => saved,
        None => fetch(cx, q, deadline, out)?,
    };
    let pool = crate::publish::keep(
        source_snapshot::write(cx, &key, &saved, &saved.sources, deadline),
        out,
    )?;
    out.dropped = saved.dropped;
    out.sources = saved.sources;
    out.problems = saved.problems;
    let mut body = saved.body;
    if saved.complete {
        slice(&mut body, q);
    }
    for row in &body.items {
        out.next.push(crate::next::Step::new(
            StepKind::Disambiguate,
            &[super::next::identity(&row.id)],
            run(Request::GetPathway(PathwayGet::new(row.id.clone()))),
        ));
    }
    if let (Some(offset), Some(pool)) = (body.page.next_offset, pool) {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &super::next::search(q),
            run(Request::SearchPathway(q.at_offset(offset).with_pool(pool))),
        ));
    }
    crate::status::search(out);
    out.body = body.with_query(q.clone());
    Ok(())
}
fn fetch(
    cx: &Ctx<'_>,
    q: &PathwaySearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Saved, RequestError> {
    if q.source() == biomcp_types::PathwaySource::Wikipathways {
        return super::wikipathways::headers(cx, q, deadline, out);
    }
    let mut body = page(q);
    let complete = q.top_level() || q.gene().is_some();
    if q.gene().is_some() {
        let mapped = super::gene::headers(cx, q, deadline, out)?;
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        body.items = mapped.value.into();
    } else {
        let leg = cx
            .fetch
            .leg(&reactome::SPEC, deadline)
            .fresh(cx.fresh || q.fresh());
        let reply = if q.top_level() {
            reactome::top_level(&leg).map(|rows| reactome::Page {
                total: rows.value.len() as u64,
                consumed: rows.value.len() as u32,
                rows,
            })
        } else {
            reactome::search(&leg, q)
        };
        out.sources.push(status::source(
            Source::Reactome,
            leg.finish(),
            reply.as_ref().err().copied(),
        ));
        let reply = reply?;
        out.include_refusals(reply.rows.refusals);
        out.problems.extend(reply.rows.problems);
        body.items = reply.rows.value.into();
        body.page.total = reply.total;
        let next = q.offset().saturating_add(reply.consumed);
        body.page.next_offset =
            (reply.consumed > 0 && u64::from(next) < reply.total).then_some(next);
    }
    if complete {
        body.page.total = body.items.len() as u64;
        body.page.total_kind = PathwayTotalKind::Observed;
    }
    body.page.returned = body.items.len() as u32;
    Ok(Saved {
        dropped: out.dropped,
        body,
        complete,
        catalog_revision: None,
        sources: out.sources.clone(),
        problems: out.problems.clone(),
    })
}
fn slice(body: &mut PathwayPage, q: &PathwaySearch) {
    body.items = body
        .items
        .iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .cloned()
        .collect();
    body.page.offset = q.offset();
    body.page.returned = body.items.len() as u32;
    let next = q.offset().saturating_add(body.page.returned);
    body.page.next_offset = (u64::from(next) < body.page.total).then_some(next);
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Saved {
    pub dropped: bool,
    pub body: PathwayPage,
    pub complete: bool,
    pub catalog_revision: Option<biodata::Text>,
    pub sources: Vec<biomcp_types::SourceOutcome>,
    pub problems: Vec<biomcp_types::Problem>,
}
