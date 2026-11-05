//! Fetch one complete NCI observation for retained local search pages.
use crate::source_snapshot as snapshot;
use crate::{Action, Ctx, Outcome, status};
use biomcp_adapters::nci;
use biomcp_fetch::{Deadline, Leg, RequestError, SourceError};
use biomcp_types::{Reason, ReplyData, Source, StepKind, UsageError};
use serde::{Serialize, de::DeserializeOwned};

type Adapter<Q, T> = fn(&Leg<'_>, &Q) -> Result<biomcp_adapters::Mapped<Vec<T>>, RequestError>;

pub(crate) trait Query: Serialize {
    fn snapshot_query(&self) -> Self;
    fn pool(&self) -> Option<&biodata::Text>;
    fn restart_action(&self) -> Action;
}

pub(crate) fn search<Q: Query, T: Serialize + DeserializeOwned>(
    cx: &Ctx<'_>,
    q: &Q,
    adapter: Adapter<Q, T>,
    namespace: &str,
    too_broad: UsageError,
    mut out: Outcome,
    page: impl FnOnce(&Q, snapshot::Saved<T>, Option<biodata::Text>, &mut Outcome),
) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    if let Err(error) = execute(cx, q, adapter, namespace, deadline, &mut out, page) {
        if out.sources.is_empty() {
            out.sources.push(biomcp_adapters::registry::outcome(
                Source::Nci,
                if status::reason(error) == Some(Reason::Disabled) {
                    biomcp_types::SourceStatus::Skipped(status::reason(error))
                } else {
                    biomcp_types::SourceStatus::Failed(status::reason(error))
                },
            ));
        }
        if status::reason(error) == Some(Reason::TooLarge) {
            out.body = ReplyData::Usage(too_broad);
        }
        out.local = status::local(error);
    }
    if out.sources.iter().any(|s| s.reason == Some(Reason::NoKey)) {
        out.next.insert(
            0,
            crate::next::Step::new(
                StepKind::Key,
                &[biomcp_types::Source::Nci],
                Action::KeyHelp(biomcp_fetch::SecretName::NciApiKey),
            ),
        );
    }
    command.finish(deadline.remaining());
    out
}
fn execute<Q: Query, T: Serialize + DeserializeOwned>(
    cx: &Ctx<'_>,
    q: &Q,
    adapter: Adapter<Q, T>,
    namespace: &str,
    deadline: Deadline<'_>,
    out: &mut Outcome,
    page: impl FnOnce(&Q, snapshot::Saved<T>, Option<biodata::Text>, &mut Outcome),
) -> Result<(), RequestError> {
    if cx.fetch.disabled(Source::Nci) {
        return Err(SourceError::disabled(Source::Nci).into());
    }
    let saved = snapshot::load(cx, namespace, &q.snapshot_query(), q.pool(), deadline)?;
    if q.pool().is_some() && !cx.fresh && saved.is_none() {
        out.body = ReplyData::PoolError { mismatch: false };
        out.next.push(crate::next::Step::new(
            StepKind::Restart,
            &[biomcp_types::Source::Nci],
            q.restart_action(),
        ));
        return Ok(());
    }
    let saved = match saved {
        Some(saved) => saved,
        None => {
            let leg = cx.fetch.leg(&nci::SPEC, deadline).fresh(cx.fresh);
            let reply = adapter(&leg, q);
            out.sources.push(status::source(
                Source::Nci,
                leg.finish(),
                reply.as_ref().err().copied(),
            ));
            let reply = reply?;
            let mut sources = std::mem::take(&mut out.sources);
            if let Some(source) = sources.first_mut() {
                source.count = Some(reply.value.len() as u64);
                source.retrieved_count = Some(reply.value.len() as u32);
            }
            snapshot::Saved {
                dropped: reply.refusals == biomcp_adapters::Refusal::Refused,
                items: reply.value,
                sources,
                problems: reply.problems,
            }
        }
    };
    let pool = if let Some(pool) = q.pool().filter(|_| !cx.fresh) {
        Some(pool.clone())
    } else {
        crate::publish::keep(
            snapshot::save(cx, namespace, &q.snapshot_query(), deadline, &saved),
            out,
        )?
    };
    page(q, saved, pool, out);
    Ok(())
}
