//! Organization discovery pages one complete NCI observation locally.
use crate::source_snapshot as snapshot;
use crate::{Action, Ctx, Outcome, status};
use biomcp_adapters::nci;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    OrganizationPage, OrganizationSearch, Reason, ReplyData, Request, Source, StepKind,
};
/// Find published organizations without assigning identities to names.
pub(crate) fn search(cx: &Ctx<'_>, q: &OrganizationSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = empty(q);
    if let Err(error) = execute(cx, q, deadline, &mut out) {
        if out.sources.is_empty() {
            out.sources.push(biomcp_adapters::registry::outcome(
                Source::Nci,
                if status::reason(error) == Some(Reason::Disabled) {
                    biomcp_types::SourceState::Skipped
                } else {
                    biomcp_types::SourceState::Failed
                },
                status::reason(error),
            ));
        }
        if status::reason(error) == Some(Reason::TooLarge) {
            out.body = ReplyData::Usage(biomcp_types::UsageError::OrganizationTooBroad);
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
fn execute(
    cx: &Ctx<'_>,
    q: &OrganizationSearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if cx.fetch.disabled(Source::Nci) {
        return Err(SourceError::disabled(Source::Nci).into());
    }
    let saved = snapshot::load(
        cx,
        "biomcp.organization-snapshot/2",
        &q.snapshot_query(),
        q.pool(),
        deadline,
    )?;
    if q.pool().is_some() && !cx.fresh && saved.is_none() {
        out.body = ReplyData::PoolError { mismatch: false };
        out.next.push(crate::next::Step::new(
            StepKind::Restart,
            &[biomcp_types::Source::Nci],
            run(q.restart()),
        ));
        return Ok(());
    }
    let saved = match saved {
        Some(saved) => saved,
        None => {
            let leg = cx.fetch.leg(&nci::SPEC, deadline).fresh(cx.fresh);
            let reply = nci::organization::search(&leg, q);
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
            snapshot::save(
                cx,
                "biomcp.organization-snapshot/2",
                &q.snapshot_query(),
                deadline,
                &saved,
            ),
            out,
        )?
    };
    page(q, saved, pool, out);
    Ok(())
}
fn page(
    q: &OrganizationSearch,
    saved: snapshot::Saved<biomcp_types::Organization>,
    pool: Option<biodata::Text>,
    out: &mut Outcome,
) {
    let total = saved.items.len() as u64;
    let items: Box<[_]> = saved
        .items
        .into_iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .collect();
    let returned = items.len() as u32;
    let next = q.offset().saturating_add(returned);
    if let Some(pool) = pool.filter(|_| returned > 0 && u64::from(next) < total) {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::Nci],
            run(q.at_page(next, pool)),
        ));
    }
    out.page = Some(biomcp_types::PageInfo {
        offset: q.offset(),
        returned,
        total,
        pool: None,
        truncated: None,
    });
    out.sources = saved.sources;
    out.problems = saved.problems;
    out.dropped = saved.dropped;
    status::search(out);
    out.body = (OrganizationPage { items }).with_query(q.clone());
}
fn empty(q: &OrganizationSearch) -> Outcome {
    crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: (OrganizationPage {
                items: Box::default(),
            })
            .with_query(q.clone()),
            page: None,
            sources: Vec::new(),
            problems: Vec::new(),
            next: Vec::new(),
            local: None,
        },
    )
}
fn run(q: OrganizationSearch) -> Action {
    Action::Run {
        request: Request::SearchOrganization(q),
        fresh: false,
    }
}
