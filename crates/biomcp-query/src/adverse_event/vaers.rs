//! One native table per request, with provider-free retained continuation.
use crate::{Ctx, Outcome, status, store::Store};
use biodata::Text;
use biomcp_adapters::vaers;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    AdverseEventSearch, PageInfo, Reason, ReplyData, Request, Source, SourceOutcome, StepKind,
    VaersSection, VaersSummary,
};
use sha2::{Digest, Sha256};
type Saved = (VaersSummary, Vec<SourceOutcome>);

pub(super) fn search(cx: &Ctx<'_>, q: &AdverseEventSearch) -> Outcome {
    let mut out = super::unsupported();
    out.body = biomcp_types::VaersSummary::into_reply(q.clone(), None);
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    if let Err(error) = read(cx, q, deadline, &mut out) {
        out.local = status::local(error);
    }
    command.finish(deadline.remaining());
    out
}
fn key(q: &AdverseEventSearch) -> Vec<u8> {
    Sha256::digest(
        format!(
            "biomcp.vaers_snapshot/1\0{}\0{}",
            q.vaccine().unwrap_or(""),
            q.vaers_section().code()
        )
        .as_bytes(),
    )
    .to_vec()
}
fn expired(q: &AdverseEventSearch, out: &mut Outcome) {
    out.body = ReplyData::PoolError { mismatch: false };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[biomcp_types::Source::Vaers],
        Request::SearchAdverseEvent(q.restart()),
        false,
    ));
}
fn read(
    cx: &Ctx<'_>,
    q: &AdverseEventSearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&vaers::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::Vaers) {
        let error = biomcp_fetch::SourceError {
            source: Source::Vaers,
            reason: Reason::Disabled,
            status: None,
            retry_after: None,
        }
        .into();
        out.sources
            .push(status::source(Source::Vaers, leg.finish(), Some(error)));
        return Err(error);
    }
    if q.offset() > 0 && q.snapshot().is_none() {
        expired(q, out);
        return Ok(());
    }
    let (saved, snapshot) = if let Some(snapshot) = q.snapshot() {
        let store = Store::open(cx.fetch.cache_access()?, deadline)?;
        let Some(payload) =
            store.source_snapshot(snapshot.as_str(), &key(q), |s| cx.fetch.disabled(s))?
        else {
            expired(q, out);
            return Ok(());
        };
        let saved: Saved = serde_json::from_str(&payload).map_err(|_| CacheError::Snapshot)?;
        (saved, snapshot.clone())
    } else {
        let reply = vaers::read(&leg, q.vaccine().unwrap_or(""), q.vaers_section());
        let error = reply.as_ref().err().copied();
        out.sources
            .push(status::source(Source::Vaers, leg.finish(), error));
        let saved = (reply?, out.sources.clone());
        let payload = serde_json::to_string(&saved).map_err(|_| CacheError::Snapshot)?;
        let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
        let snapshot = store.save_source_snapshot(&key(q), &payload, &out.sources)?;
        (saved, snapshot)
    };
    out.sources = saved.1;
    {
        let unavailable = out
            .sources
            .iter()
            .any(|s| s.state == biomcp_types::SourceState::Stale);
        crate::status::item(out, true, false, unavailable);
    };
    render(q, saved.0, snapshot, out);
    Ok(())
}
fn render(q: &AdverseEventSearch, mut table: VaersSummary, snapshot: Text, out: &mut Outcome) {
    let total = table.rows.len() as u64;
    table.rows = table
        .rows
        .iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .cloned()
        .collect();
    let returned = table.rows.len() as u32;
    out.page = Some(PageInfo {
        offset: q.offset(),
        returned,
        total,
        pool: None,
        truncated: None,
    });
    let offset = q.offset().saturating_add(returned);
    if returned > 0 && u64::from(offset) < total {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[biomcp_types::Source::Vaers],
            Request::SearchAdverseEvent(q.with_snapshot(snapshot).at_offset(offset)),
            false,
        ));
    }
    for section in [
        VaersSection::Symptoms,
        VaersSection::Seriousness,
        VaersSection::Age,
    ] {
        if section != q.vaers_section() {
            out.next.push(crate::next::Step::run(
                StepKind::Section,
                &[biomcp_types::Source::Vaers],
                Request::SearchAdverseEvent(q.with_vaers_section(section)),
                false,
            ));
        }
    }
    out.body = table.with_query(q.clone());
}
