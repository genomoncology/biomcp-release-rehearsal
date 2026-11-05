//! Separate source legs take turns without title joins or score blending.
use super::{empty, run};
use crate::{Ctx, Outcome, source_snapshot, status};
use biomcp_adapters::{reactome, wikipathways};
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PathwayGet, PathwayHeader, PathwayPage, PathwayPageInfo, PathwaySearch, PathwaySource,
    PathwayTotalKind, Problem, ReplyData, Request, Source, SourceOutcome, StepKind,
};
// Queued whole rows, next native position, source total and catalog content revision.
type LegState = (Vec<PathwayHeader>, Option<u32>, u64, Option<biodata::Text>);
type Saved = (
    PathwayPage,
    Vec<LegState>,
    usize,
    Vec<SourceOutcome>,
    Vec<Problem>,
    bool,
);
fn key(q: &PathwaySearch) -> Result<Vec<u8>, RequestError> {
    source_snapshot::key("pathway-union", &q.restart())
}
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
pub(super) fn search(cx: &Ctx<'_>, q: &PathwaySearch) -> Outcome {
    let mut out = empty((page(q)).with_query(q.clone()));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    if let Err(error) = execute(cx, q, deadline, &mut out) {
        out.local = status::local(error);
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
    // Reconstructing a mixed admitted offset would require walking prior native pages.
    // Continue only from retained source positions; fresh restarts begin at zero.
    if q.offset() > 0 && (q.pool().is_none() || cx.fresh || q.fresh()) {
        expired(q, out);
        return Ok(());
    }
    let identity = key(q)?;
    let loaded = load(cx, q, deadline, &identity, out)?;
    if matches!(out.body, ReplyData::PoolError { .. }) {
        return Ok(());
    }
    let replay = loaded
        .as_ref()
        .is_some_and(|s| s.0.page.offset == q.offset());
    let mut saved = match loaded {
        Some(saved) if replay || saved.0.page.next_offset == Some(q.offset()) => saved,
        Some(_) => {
            expired(q, out);
            return Ok(());
        }
        None => initial(cx, q, deadline, out),
    };
    if !replay {
        advance(cx, q, deadline, &mut saved, out);
    }
    let pool = if replay {
        Some(q.pool().cloned().ok_or(CacheError::Internal)?)
    } else {
        crate::publish::keep(
            source_snapshot::write(cx, &identity, &saved, &saved.3, deadline),
            out,
        )?
    };
    present(q, saved, pool, out);
    Ok(())
}
fn load(
    cx: &Ctx<'_>,
    q: &PathwaySearch,
    deadline: Deadline<'_>,
    identity: &[u8],
    out: &mut Outcome,
) -> Result<Option<Saved>, RequestError> {
    let Some(pool) = q.pool().filter(|_| !(cx.fresh || q.fresh())) else {
        return Ok(None);
    };
    let saved: Option<Saved> = source_snapshot::read(cx, identity, pool.as_str(), deadline)?;
    let Some(saved) = saved else {
        expired(q, out);
        return Ok(None);
    };
    if saved.1.len() != q.sources().len() {
        return Err(CacheError::Internal.into());
    }
    Ok(Some(saved))
}
fn present(q: &PathwaySearch, saved: Saved, pool: Option<biodata::Text>, out: &mut Outcome) {
    out.sources = saved.3;
    out.problems = saved.4;
    for row in &saved.0.items {
        out.next.push(crate::next::Step::new(
            StepKind::Disambiguate,
            &[super::next::identity(&row.id)],
            run(Request::GetPathway(PathwayGet::new(row.id.clone()))),
        ));
    }
    if let (Some(offset), Some(pool)) = (saved.0.page.next_offset, pool) {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &super::next::search(q),
            run(Request::SearchPathway(q.at_offset(offset).with_pool(pool))),
        ));
    }
    out.dropped = saved.5;
    status::search(out);
    out.body = (saved.0).with_query(q.clone());
}
fn expired(q: &PathwaySearch, out: &mut Outcome) {
    out.body = ReplyData::PoolError { mismatch: false };
    out.next.push(crate::next::Step::new(
        StepKind::Restart,
        &super::next::search(q),
        run(Request::SearchPathway(q.restart())),
    ));
}
fn initial(cx: &Ctx<'_>, q: &PathwaySearch, deadline: Deadline<'_>, out: &mut Outcome) -> Saved {
    let mut saved = (page(q), vec![], 0, vec![], vec![], false);
    for source in q.sources() {
        let leg = cx.fetch.leg(
            if *source == PathwaySource::Reactome {
                &reactome::SPEC
            } else {
                &wikipathways::SPEC
            },
            deadline,
        );
        if *source == PathwaySource::Wikipathways {
            let reply = wikipathways::search(&leg, q.query().map_or("", |v| v.as_str()));
            let error = reply.as_ref().err().copied();
            out.local = out.local.or_else(|| error.and_then(status::local));
            saved
                .3
                .push(status::source(Source::WikiPathways, leg.finish(), error));
            let state = match reply {
                Ok((rows, info)) => {
                    saved.5 |= rows.refusals == biomcp_adapters::Refusal::Refused;
                    let total = rows.value.len() as u64;
                    saved.4.extend(rows.problems);
                    (
                        rows.value,
                        None,
                        total,
                        info.content_revision.and_then(|s| s.parse().ok()),
                    )
                }
                Err(_) => (vec![], None, 0, None),
            };
            saved.1.push(state);
        } else {
            // Initial native admission happens in advance, once per source per command.
            drop(leg);
            saved.1.push((vec![], Some(0), 0, None));
            saved.3.push(biomcp_adapters::registry::outcome(
                Source::Reactome,
                biomcp_types::SourceStatus::Ok,
            ));
        }
    }
    saved
}
fn refill(
    cx: &Ctx<'_>,
    q: &PathwaySearch,
    deadline: Deadline<'_>,
    saved: &mut Saved,
    out: &mut Outcome,
) {
    for (index, state) in saved.1.iter_mut().enumerate() {
        if !state.0.is_empty() {
            continue;
        }
        let Some(offset) = state.1 else {
            continue;
        };
        let leg = cx
            .fetch
            .leg(&reactome::SPEC, deadline)
            .fresh(cx.fresh || q.fresh());
        let reply = reactome::search(&leg, &q.reactome_leg().at_offset(offset));
        let error = reply.as_ref().err().copied();
        out.local = out.local.or_else(|| error.and_then(status::local));
        if let Some(source) = saved.3.get_mut(index) {
            *source = status::source(Source::Reactome, leg.finish(), error);
        }
        match reply {
            Ok(reply) => {
                saved.5 |= reply.rows.refusals == biomcp_adapters::Refusal::Refused;
                let next = offset.saturating_add(reply.consumed);
                state.0 = reply.rows.value;
                state.1 = (reply.consumed > 0 && u64::from(next) < reply.total).then_some(next);
                state.2 = reply.total;
                saved.4.extend(reply.rows.problems);
            }
            Err(_) => state.1 = None,
        }
    }
}
fn advance(
    cx: &Ctx<'_>,
    q: &PathwaySearch,
    deadline: Deadline<'_>,
    saved: &mut Saved,
    out: &mut Outcome,
) {
    refill(cx, q, deadline, saved, out);
    let mut items = Vec::new();
    let mut misses = 0;
    while items.len() < q.limit() as usize && misses < saved.1.len() {
        let index = saved.2 % saved.1.len();
        // A shortened native page stops at its source turn, without refilling or
        // moving later catalog rows ahead of unrequested source rows.
        if saved
            .1
            .get(index)
            .is_some_and(|s| s.0.is_empty() && s.1.is_some())
        {
            break;
        }
        saved.2 = (index + 1) % saved.1.len();
        if let Some(state) = saved.1.get_mut(index)
            && !state.0.is_empty()
        {
            items.push(state.0.remove(0));
            misses = 0;
        } else {
            misses += 1;
        }
    }
    let returned = items.len() as u32;
    let continuing = saved.1.iter().any(|s| !s.0.is_empty() || s.1.is_some());
    saved.0 = PathwayPage {
        items: items.into(),
        page: PathwayPageInfo {
            offset: q.offset(),
            returned,
            total: saved.1.iter().map(|s| s.2).sum(),
            total_kind: PathwayTotalKind::Source,
            next_offset: continuing.then_some(q.offset().saturating_add(returned.max(1))),
            pool: None,
        },
    };
}
