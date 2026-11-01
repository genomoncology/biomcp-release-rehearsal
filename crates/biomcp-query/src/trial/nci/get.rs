//! NCI native snapshots use the existing trial store and provider policy admission.
use super::{ATTRIBUTION, empty, finish, page};
use crate::{Ctx, Outcome, source_snapshot, status};
use biomcp_types::ReplyBody;

use biomcp_adapters::nci;
use biomcp_fetch::{CacheError, RequestError};
use biomcp_types::{
    NciTrialCards, Reason, ReplyData, Request, Source, SourceOutcome, StepKind, TrialGet,
};
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Snapshot {
    pub(super) detail: nci::Detail,
    pub(super) fetched: Option<u64>,
    pub(super) sources: Vec<SourceOutcome>,
}

pub(in crate::trial) fn run(cx: &Ctx<'_>, q: &TrialGet) -> Outcome {
    let mut out = empty(
        (NciTrialCards {
            items: Box::default(),
            attribution: ATTRIBUTION.into(),
        })
        .with_query(q.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let result = if q.pool().is_some() && !cx.fetch.disabled(Source::Nci) {
        restore(cx, q, deadline, &mut out)
    } else {
        fetch(cx, q, deadline, &mut out)
    };
    if let Err(error) = result {
        crate::status::item(&mut out, false, true, false);
        out.local = status::local(error);
    }
    command.finish(deadline.remaining());
    out
}
fn fetch(
    cx: &Ctx<'_>,
    q: &TrialGet,
    deadline: biomcp_fetch::Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&nci::SPEC, deadline).fresh(cx.fresh);
    let rows: Vec<_> = q.input().iter().map(|id| nci::get(&leg, *id)).collect();
    let error = rows
        .iter()
        .find_map(|r| r.as_ref().err().copied().filter(|e| !absent(*e)));
    let source = status::source(Source::Nci, leg.finish(), error);
    let mut items = Vec::new();
    for (id, row) in q.input().iter().zip(rows) {
        match row {
            Ok((detail, meta)) => {
                let snapshot = Snapshot {
                    detail,
                    fetched: meta.fetched.and_then(|value| u64::try_from(value).ok()),
                    sources: vec![source.clone()],
                };
                let pool = if q.sections().is_empty() {
                    None
                } else {
                    crate::publish::keep(
                        source_snapshot::write(
                            cx,
                            &source_snapshot::key("nci-trial", id)?,
                            &snapshot,
                            &snapshot.sources,
                            deadline,
                        ),
                        out,
                    )?
                };
                items.push(page::render(q, snapshot, pool, out)?);
            }
            Err(e) => items.push(biomcp_types::NciTrialItem {
                input: *id,
                status: if absent(e) {
                    crate::status::item_status(false, false, false)
                } else {
                    crate::status::item_status(false, true, false)
                },
                card: None,
                fetched: None,
                sections: None,
                more: Box::default(),
                unsupported_sections: Box::default(),
            }),
        }
    }
    status::get(out, items.iter().map(|i| i.status));
    if let ReplyData::NciTrialCards {
        body: Some(body), ..
    } = &mut out.body
    {
        body.items = items.into();
    }
    finish(out, source, error);
    Ok(())
}
fn restore(
    cx: &Ctx<'_>,
    q: &TrialGet,
    deadline: biomcp_fetch::Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let id = q.input().first().copied().ok_or(CacheError::Internal)?;
    let snapshot: Option<Snapshot> = source_snapshot::read(
        cx,
        &source_snapshot::key("nci-trial", &id)?,
        q.pool().map_or("", |p| p.as_str()),
        deadline,
    )?;
    let Some(snapshot) = snapshot else {
        out.body = ReplyData::PoolError { mismatch: false };
        out.next.push(crate::next::Step::run(
            StepKind::Restart,
            &[biomcp_types::Source::Nci],
            Request::GetTrial(q.identity()),
            false,
        ));
        return Ok(());
    };
    if snapshot.detail.card.nct_id != id {
        return Err(CacheError::Internal.into());
    }
    if !page::valid(q, &snapshot) {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
        return Ok(());
    }
    out.sources = snapshot.sources.clone();
    let item = page::render(q, snapshot, q.pool().cloned(), out)?;
    let state = item.status;
    crate::status::get(out, [state].into_iter());
    if let ReplyData::NciTrialCards {
        body: Some(body), ..
    } = &mut out.body
    {
        body.items = Box::new([item]);
    }
    Ok(())
}
fn absent(e: RequestError) -> bool {
    matches!(e,RequestError::Source(s) if s.reason==Reason::NotFound)
}
