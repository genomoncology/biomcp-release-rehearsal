//! Exact trial cards and immutable source section continuations.
use biomcp_types::ReplyBody;
pub(in crate::trial) mod page;
use crate::{Ctx, Outcome, source_snapshot, status};
use biodata::NctId;
use biomcp_adapters::ctgov;
use biomcp_fetch::{CacheError, RequestError};
use biomcp_types::{
    ReplyData, Request, Source, SourceState, Status, StepKind, TrialCardItem, TrialCards, TrialGet,
};
/// Retrieve independent input states under one shared command deadline.
pub(crate) fn get(cx: &Ctx<'_>, query: &TrialGet) -> Outcome {
    if query.source() == biomcp_types::TrialSource::Nci {
        return super::nci::get(cx, query);
    }

    let mut out = empty(query);
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let result = if query.pool().is_some() {
        restore(cx, query, deadline, &mut out)
    } else {
        fetch(cx, query, deadline, &mut out)
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
    query: &TrialGet,
    deadline: biomcp_fetch::Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&ctgov::SPEC, deadline).fresh(cx.fresh);
    let rows: Vec<_> = query
        .input()
        .iter()
        .map(|id| ctgov::detail::get(&leg, *id))
        .collect();
    let version = rows.iter().any(Result::is_ok).then(|| ctgov::version(&leg));
    let error = rows
        .iter()
        .find_map(|r| r.as_ref().err().copied().filter(|e| !not_found(*e)))
        .or_else(|| version.as_ref().and_then(|v| v.as_ref().err().copied()));
    let source = status::source(Source::ClinicalTrialsGov, leg.finish(), error);
    out.local = error.and_then(status::local);
    out.sources.push(source.clone());
    let version = version.and_then(Result::ok);
    let processing_problems = version
        .as_ref()
        .map_or_else(Vec::new, |version| version.problems.clone());
    let source_updated = version.and_then(|version| version.value);
    let mut items = Vec::new();
    for (id, row) in query.input().iter().zip(rows) {
        match row {
            Ok((detail, metadata)) => {
                let mut own_source = source.clone();
                if own_source.state == SourceState::Failed && source_updated.is_some() {
                    own_source.state = SourceState::Ok;
                    own_source.reason = None;
                }
                let snapshot = Snapshot {
                    detail,
                    fetched: metadata.fetched.and_then(|value| u64::try_from(value).ok()),
                    version: source_updated.clone(),
                    processing_problems: processing_problems.clone(),
                    sources: vec![own_source],
                };
                let pool = if query.sections().is_empty() {
                    None
                } else {
                    Some(source_snapshot::write(
                        cx,
                        &source_snapshot::key("trial", id)?,
                        &snapshot,
                        &snapshot.sources,
                        deadline,
                    )?)
                };
                items.push(page::render(query, snapshot, pool, out)?);
            }
            Err(e) => items.push(unresolved(
                *id,
                if not_found(e) {
                    crate::status::item_status(false, false, false)
                } else {
                    crate::status::item_status(false, true, false)
                },
            )),
        }
    }
    finish(query, items, source_updated, out);
    Ok(())
}
fn restore(
    cx: &Ctx<'_>,
    query: &TrialGet,
    deadline: biomcp_fetch::Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let id = query.input().first().copied().ok_or(CacheError::Database)?;
    if cx.fetch.disabled(Source::ClinicalTrialsGov) {
        let leg = cx.fetch.leg(&ctgov::SPEC, deadline);
        let result = ctgov::detail::get(&leg, id);
        let error = result.err();
        out.sources.push(status::source(
            Source::ClinicalTrialsGov,
            leg.finish(),
            error,
        ));
        finish(
            query,
            vec![unresolved(
                id,
                crate::status::item_status(false, true, false),
            )],
            None,
            out,
        );
        return Ok(());
    }
    let snapshot: Option<Snapshot> = source_snapshot::read(
        cx,
        &source_snapshot::key("trial", &id)?,
        query.pool().map_or("", |p| p.as_str()),
        deadline,
    )?;
    let Some(snapshot) = snapshot else {
        out.body = ReplyData::PoolError { mismatch: false };
        out.next.push(crate::next::Step::run(
            StepKind::Restart,
            &[query.source().source()],
            Request::GetTrial(query.identity()),
            false,
        ));
        return Ok(());
    };
    if snapshot.detail.card.nct_id != id {
        return Err(CacheError::Database.into());
    }
    if !page::valid(query, &snapshot) {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
        return Ok(());
    }
    let version = snapshot.version.clone();
    out.sources = snapshot.sources.clone();
    let item = page::render(query, snapshot, query.pool().cloned(), out)?;
    finish(query, vec![item], version, out);
    Ok(())
}
fn not_found(error: RequestError) -> bool {
    matches!(error,RequestError::Source(e) if e.reason==biomcp_types::Reason::NotFound)
}
fn unresolved(input: NctId, status: Status) -> TrialCardItem {
    TrialCardItem {
        input,
        status,
        card: None,
        fetched: None,
        sections: None,
        more: Box::default(),
    }
}
fn finish(
    query: &TrialGet,
    items: Vec<TrialCardItem>,
    source_updated: Option<biomcp_types::SourceTimestamp>,
    out: &mut Outcome,
) {
    status::get(out, items.iter().map(|i| i.status));
    out.body = (TrialCards {
        items: items.into(),
        source_updated: source_updated.map(Into::into),
        attribution: "ClinicalTrials.gov; registry protocol text, no eligibility assessment".into(),
    })
    .with_query(query.clone());
}
fn empty(query: &TrialGet) -> Outcome {
    crate::status::empty(
        (TrialCards {
            items: Box::default(),
            source_updated: None,
            attribution: "ClinicalTrials.gov".into(),
        })
        .with_query(query.clone()),
    )
}
fn partial(snapshot: &Snapshot) -> bool {
    snapshot.detail.refusals == biomcp_adapters::Refusal::Refused
        || snapshot.sources.iter().any(|s| {
            matches!(
                s.state,
                SourceState::Failed | SourceState::Stale | SourceState::Syncing
            )
        })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Snapshot {
    pub(super) detail: biomcp_adapters::ctgov::detail::Detail,
    pub(super) fetched: Option<u64>,
    pub(super) version: Option<biomcp_types::SourceTimestamp>,
    pub(super) processing_problems: Vec<biomcp_types::Problem>,
    pub(super) sources: Vec<biomcp_types::SourceOutcome>,
}
