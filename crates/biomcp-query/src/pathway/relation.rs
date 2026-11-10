use super::empty;
use crate::{Ctx, Outcome, source_snapshot, status};
use biodata::{PathwayId, Text};
use biomcp_adapters::reactome;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::ReplyBody;
use biomcp_types::{
    PathwayEventsQuery, PathwayGenesQuery, PathwayHeader, PathwayPageInfo, PathwayTotalKind,
    Reason, ReplyData, Request, Source, StepKind,
};
pub(super) enum Echo {
    Genes(PathwayGenesQuery),
    Events(PathwayEventsQuery),
}
pub(super) struct Selection<'a> {
    pub echo: Echo,
    pub id: &'a PathwayId,
    pub genes: bool,
    pub details: bool,
    pub limit: u32,
    pub offset: u32,
    pub pool: Option<&'a Text>,
    pub fresh: bool,
    pub header: Option<PathwayHeader>,
}
impl Selection<'_> {
    fn source(&self) -> Source {
        if matches!(self.id, PathwayId::Wikipathways(_)) {
            Source::WikiPathways
        } else {
            Source::Reactome
        }
    }
}
/// Page retained source membership with checked gene pivots.
pub(crate) fn genes(cx: &Ctx<'_>, q: &PathwayGenesQuery) -> Outcome {
    execute(
        cx,
        Selection {
            echo: Echo::Genes(q.clone()),
            id: q.id(),
            genes: true,
            details: q.details(),
            limit: q.limit(),
            offset: q.offset(),
            pool: q.pool(),
            fresh: cx.fresh,
            header: None,
        },
    )
}
/// Page retained source event objects and unresolved integer references.
pub(crate) fn events(cx: &Ctx<'_>, q: &PathwayEventsQuery) -> Outcome {
    execute(
        cx,
        Selection {
            echo: Echo::Events(q.clone()),
            id: q.id(),
            genes: false,
            details: false,
            limit: q.limit(),
            offset: q.offset(),
            pool: q.pool(),
            fresh: cx.fresh,
            header: None,
        },
    )
}
fn execute(cx: &Ctx<'_>, q: Selection<'_>) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let out = read(cx, q, deadline);
    command.finish(deadline.remaining());
    out
}
pub(super) fn read(cx: &Ctx<'_>, q: Selection<'_>, deadline: Deadline<'_>) -> Outcome {
    let mut out = empty(match &q.echo {
        Echo::Genes(query) => biomcp_types::PathwayGenes::into_reply(query.clone(), None),
        Echo::Events(query) => biomcp_types::PathwayEvents::into_reply(query.clone(), None),
    });
    if let Err(error) = read_inner(cx, &q, deadline, &mut out) {
        out.local = status::local(error);
        crate::status::item(&mut out, false, true, false);
    }
    out
}
fn restart(q: &Selection<'_>, out: &mut Outcome) {
    out.body = ReplyData::PoolError { mismatch: false };
    let request = if q.genes {
        PathwayGenesQuery::new(q.id.clone(), q.limit)
            .map(|query| {
                if q.details {
                    query.with_details()
                } else {
                    query
                }
            })
            .map(Request::PathwayGenes)
    } else {
        PathwayEventsQuery::new(q.id.clone(), q.limit).map(Request::PathwayEvents)
    };
    if let Ok(request) = request {
        out.next.push(crate::next::Step::new(
            StepKind::Restart,
            &[q.source()],
            super::run(request),
        ));
    }
}
fn read_inner(
    cx: &Ctx<'_>,
    q: &Selection<'_>,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let source = q.source();
    if cx.fetch.disabled(source) {
        let error: RequestError = SourceError::disabled(source).into();
        out.sources.push(biomcp_adapters::registry::outcome(
            source,
            biomcp_types::SourceStatus::Skipped(Some(Reason::Disabled)),
        ));
        return Err(error);
    }
    if q.pool.is_some() && q.fresh || q.offset > 0 && q.pool.is_none() {
        restart(q, out);
        return Ok(());
    }
    let loaded = if q.fresh {
        None
    } else {
        source_snapshot::find::<Saved>(
            cx,
            &source_snapshot::key("pathway-relation", &(q.id, q.genes))?,
            q.pool,
            deadline,
        )?
        .0
    };
    let (saved, pool) = match loaded {
        Some((saved, pool)) => (saved, Some(pool)),
        None if q.pool.is_some() => {
            restart(q, out);
            return Ok(());
        }
        None => {
            let saved = fetch(cx, q, deadline, out)?;
            let pool = crate::publish::keep(
                source_snapshot::publish(
                    cx,
                    &source_snapshot::key("pathway-relation", &(q.id, q.genes))?,
                    &saved,
                    saved
                        .genes
                        .as_ref()
                        .map(|g| g.items.len())
                        .or_else(|| saved.events.as_ref().map(|e| e.items.len()))
                        .unwrap_or(0) as u32,
                    &saved.sources,
                    None,
                    deadline,
                ),
                out,
            )?;
            (saved, pool)
        }
    };
    out.dropped = saved.dropped;
    out.sources = saved.sources;
    out.problems = saved.problems;
    match (&q.echo, saved.genes, saved.events) {
        (Echo::Genes(query), Some(body), _) => {
            present_genes(cx, q, query, deadline, body, pool.map(|p| p.info), out)
        }
        (Echo::Events(query), _, Some(body)) => {
            present_events(q, query, body, pool.map(|p| p.info), out)
        }
        _ => {}
    }
    Ok(())
}
fn present_genes(
    cx: &Ctx<'_>,
    q: &Selection<'_>,
    query: &PathwayGenesQuery,
    deadline: Deadline<'_>,
    mut body: biomcp_types::PathwayGenes,
    pool: Option<biomcp_types::PoolInfo>,
    out: &mut Outcome,
) {
    body.items = body
        .items
        .iter()
        .skip(q.offset as usize)
        .take(q.limit as usize)
        .cloned()
        .collect();
    body.page = position(body.page, q, body.items.len(), pool);
    if !q.details {
        for row in &mut body.items {
            row.participants = None;
        }
    }
    let resolve = if matches!(q.id, PathwayId::Wikipathways(_)) {
        super::gpml_membership::resolve
    } else {
        super::membership::resolve
    };
    resolve(
        cx,
        &mut body.items,
        deadline,
        &mut out.sources,
        &mut out.problems,
    );
    super::relation_steps::genes(&body, q.limit, q.details, &mut out.next, &mut out.problems);
    {
        let absent = body.page.total == 0;
        crate::status::item(out, !absent, false, false);
    }
    out.body = body.with_query(query.clone());
}
fn present_events(
    q: &Selection<'_>,
    query: &PathwayEventsQuery,
    mut body: biomcp_types::PathwayEvents,
    pool: Option<biomcp_types::PoolInfo>,
    out: &mut Outcome,
) {
    body.items = body
        .items
        .iter()
        .skip(q.offset as usize)
        .take(q.limit as usize)
        .cloned()
        .collect();
    body.page = position(body.page, q, body.items.len(), pool);
    super::relation_steps::events(&body, q.limit, &mut out.next, &mut out.problems);
    {
        let absent = body.page.total == 0;
        crate::status::item(out, !absent, false, false);
    }
    out.body = body.with_query(query.clone());
}

fn position(
    mut page: PathwayPageInfo,
    q: &Selection<'_>,
    returned: usize,
    pool: Option<biomcp_types::PoolInfo>,
) -> PathwayPageInfo {
    page.offset = q.offset;
    page.returned = returned as u32;
    let next = q.offset.saturating_add(q.limit);
    page.next_offset = (pool.is_some() && u64::from(next) < page.total).then_some(next);
    page.pool = pool;
    page
}
fn fetch(
    cx: &Ctx<'_>,
    q: &Selection<'_>,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Saved, RequestError> {
    if matches!(q.id, PathwayId::Wikipathways(_)) {
        return super::gpml_relation::fetch(cx, q, deadline, out);
    }
    let PathwayId::Reactome(id) = q.id else {
        return Err(biomcp_fetch::InputError::Serialization.into());
    };
    let leg = cx.fetch.leg(&reactome::SPEC, deadline).fresh(q.fresh);
    let reply = (|| {
        let header = match &q.header {
            Some(header) => header.clone(),
            None => {
                let mapped = reactome::card(&leg, id, false)?;
                out.include_refusals(mapped.refusals);
                out.problems.extend(mapped.problems);
                let card = mapped.value.ok_or(SourceError {
                    source: Source::Reactome,
                    reason: Reason::NotFound,
                    status: Some(404),
                    tls_cause: None,
                    retry_after: None,
                })?;
                PathwayHeader {
                    id: card.pathway.id,
                    name: card.pathway.name,
                    species: card.pathway.species,
                    version: card.pathway.version,
                    revision: None,
                }
            }
        };
        let mut saved = Saved {
            dropped: out.dropped,
            genes: None,
            events: None,
            sources: Vec::new(),
            problems: Vec::new(),
        };
        if q.genes {
            let rows = reactome::members(&leg, id)?;
            saved.dropped |= rows.refusals == biomcp_adapters::Refusal::Refused;
            saved.problems = rows.problems;
            saved.genes = Some(biomcp_types::PathwayGenes {
                header,
                page: full_page(rows.value.items.len(), PathwayTotalKind::DistinctReferences),
                items: rows.value.items.into(),
                reference_occurrences: rows.value.reference_occurrences,
                physical_entities: rows.value.physical_entities,
            });
        } else {
            let rows = reactome::events(&leg, id)?;
            saved.dropped |= rows.refusals == biomcp_adapters::Refusal::Refused;
            saved.problems = rows.problems;
            saved.events = Some(biomcp_types::PathwayEvents {
                header,
                page: full_page(rows.value.len(), PathwayTotalKind::Observed),
                items: rows.value.into(),
            });
        }
        Ok::<_, RequestError>(saved)
    })();
    out.sources.push(status::source(
        Source::Reactome,
        leg.finish(),
        reply.as_ref().err().copied(),
    ));
    let mut saved = reply?;
    saved.sources = out.sources.clone();
    saved.problems.extend(out.problems.clone());
    Ok(saved)
}
fn full_page(size: usize, kind: PathwayTotalKind) -> PathwayPageInfo {
    PathwayPageInfo {
        offset: 0,
        returned: size as u32,
        total: size as u64,
        total_kind: kind,
        next_offset: None,
        pool: None,
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Saved {
    pub dropped: bool,
    pub genes: Option<biomcp_types::PathwayGenes>,
    pub events: Option<biomcp_types::PathwayEvents>,
    pub sources: Vec<biomcp_types::SourceOutcome>,
    pub problems: Vec<biomcp_types::Problem>,
}

impl crate::source_snapshot::Observation for Saved {
    fn whole(&self) -> bool {
        !self.dropped && crate::source_snapshot::sources_whole(&self.sources)
    }
}
