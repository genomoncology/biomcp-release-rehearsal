//! NCI native snapshots use the existing trial store and provider policy admission.
use super::{ATTRIBUTION, empty, finish, page};
use crate::{Ctx, Outcome, status, store::Store};
use biomcp_types::ReplyBody;

use biomcp_adapters::nci;
use biomcp_fetch::{CacheError, RequestError};
use biomcp_types::{
    NciTrialCards, Reason, ReplyData, Request, Source, SourceOutcome, StepKind, TrialGet,
};
use sha2::{Digest, Sha256};
pub(super) struct Snapshot {
    pub(super) detail: nci::Detail,
    pub(super) fetched: Option<u64>,
    pub(super) sources: Vec<SourceOutcome>,
}
fn key(id: biodata::NctId) -> Vec<u8> {
    Sha256::digest(format!("biomcp.nci_trial_snapshot/3\0{id}").as_bytes()).to_vec()
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
    let mut store = if !q.sections().is_empty() && rows.iter().any(Result::is_ok) {
        Some(Store::open(cx.fetch.cache_access()?, deadline)?)
    } else {
        None
    };
    let mut items = Vec::new();
    for (id, row) in q.input().iter().zip(rows) {
        match row {
            Ok((detail, meta)) => {
                let snapshot = Snapshot {
                    detail,
                    fetched: meta.fetched.and_then(|value| u64::try_from(value).ok()),
                    sources: vec![source.clone()],
                };
                let pool = if let Some(store) = store.as_mut() {
                    Some(store.save_source_snapshot(
                        &key(*id),
                        &snapshot.encode()?,
                        &snapshot.sources,
                    )?)
                } else {
                    None
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
    let id = q.input().first().copied().ok_or(CacheError::Database)?;
    let store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let payload = store.source_snapshot(q.pool().map_or("", |p| p.as_str()), &key(id), |s| {
        cx.fetch.disabled(s)
    })?;
    let Some(payload) = payload else {
        out.body = ReplyData::PoolError { mismatch: false };
        out.next.push(crate::next::Step::run(
            StepKind::Restart,
            &[biomcp_types::Source::Nci],
            Request::GetTrial(q.identity()),
            false,
        ));
        return Ok(());
    };
    let snapshot = Snapshot::decode(&payload)?;
    if snapshot.detail.card.nct_id != id {
        return Err(CacheError::Database.into());
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

impl Snapshot {
    fn encode(&self) -> Result<String, CacheError> {
        serde_json::to_string(&(&self.detail, self.fetched, &self.sources))
            .map_err(|_| CacheError::Snapshot)
    }
    fn decode(payload: &str) -> Result<Self, CacheError> {
        let (detail, fetched, sources) =
            serde_json::from_str(payload).map_err(|_| CacheError::Snapshot)?;
        Ok(Self {
            detail,
            fetched,
            sources,
        })
    }
}
