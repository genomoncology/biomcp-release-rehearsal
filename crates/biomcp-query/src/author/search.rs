use super::{empty, retry, run};
use crate::{Ctx, Outcome, source_snapshot as snapshot, status};
use biomcp_adapters::semantic_scholar::{self, author};
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    AuthorGet, AuthorPage, AuthorSearch, PageInfo, Reason, ReplyData, Request, Source, SourceState,
    StepKind,
};
type Saved = super::NativePages<(author::Page, bool)>;
/// Preserve the source's exact query, candidate order and continuation.
pub(crate) fn search(cx: &Ctx<'_>, q: &AuthorSearch) -> Outcome {
    if q.source() == biomcp_types::AuthorSearchSource::Pubmed
        || q.source() == biomcp_types::AuthorSearchSource::Auto
            && cx.fetch.disabled(Source::SemanticScholar)
            && q.pool().is_none()
    {
        return super::literature::search(cx, q);
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = empty(biomcp_types::AuthorPage::into_reply(q.clone(), None));
    let executed = execute(cx, q, deadline, &mut out);
    if executed.is_ok() && found_none(q, &out) {
        // The open source gets a turn; both sources stay listed so the answer names who found it.
        let mut next = super::literature::search(cx, q);
        next.sources.splice(0..0, out.sources);
        command.finish(deadline.remaining());
        return next;
    }
    if let Err(error) = executed {
        out.local = status::local(error);
        if out.sources.is_empty() {
            out.sources.push(biomcp_adapters::registry::outcome(
                Source::SemanticScholar,
                biomcp_types::SourceStatus::Failed(None),
            ));
        }
    }
    let key = out
        .next
        .iter()
        .position(|step| step.0 == StepKind::Key)
        .map(|index| out.next.remove(index));
    retry(&mut out, Request::SearchAuthor(q.restart()));
    if let Some(key) = key {
        out.next.push(key);
    }
    command.finish(deadline.remaining());
    out
}
/// An empty first page from the only automatic source says nothing about the open sources.
fn found_none(q: &AuthorSearch, out: &Outcome) -> bool {
    q.source() == biomcp_types::AuthorSearchSource::Auto
        && q.pool().is_none()
        && q.offset() == 0
        && out.local.is_none()
        && out.page.as_ref().is_some_and(|page| page.total == 0)
}
fn execute(
    cx: &Ctx<'_>,
    q: &AuthorSearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if cx.fetch.disabled(Source::SemanticScholar) {
        out.sources.push(biomcp_adapters::registry::outcome(
            Source::SemanticScholar,
            biomcp_types::SourceStatus::Skipped(Some(Reason::Disabled)),
        ));
        return Err(SourceError::disabled(Source::SemanticScholar).into());
    }
    let key = snapshot::key("author-search", &(q.query(), q.source(), q.limit()))?;
    let saved = retained(cx, q, &key, deadline, out)?;
    if matches!(out.body, ReplyData::PoolError { .. }) {
        return Ok(());
    }
    let (mut saved, old_pool) = if let Some((saved, pool)) = saved {
        (saved, Some(pool))
    } else {
        (
            super::NativePages {
                pages: vec![],
                key_needed: false,
                source: biomcp_adapters::registry::outcome(
                    Source::SemanticScholar,
                    biomcp_types::SourceStatus::Ok,
                ),
            },
            None,
        )
    };
    for (page, refused) in &mut saved.pages {
        page.restore_refusals(*refused);
    }
    let cached = saved.pages.iter().any(|p| p.0.offset() == q.offset());
    if !cached {
        let leg = cx
            .fetch
            .leg(&semantic_scholar::SPEC, deadline)
            .fresh(cx.fresh);
        let result = author::search(&leg, q);
        let error = result.as_ref().err().copied();
        let source = status::source(Source::SemanticScholar, leg.finish(), error);
        out.sources.push(source.clone());
        let key_needed = source.reason == Some(Reason::RateLimited)
            && !cx
                .fetch
                .has_key(biomcp_fetch::SecretName::SemanticScholarApiKey);
        if key_needed {
            super::key_step(out);
        }
        saved.key_needed |= key_needed;
        let page = result?;
        let refused = page.refusals() == biomcp_adapters::Refusal::Refused;
        saved.pages.push((page, refused));
        status::retain_source_state(&mut saved.source, &source);
    }
    let pool = if cached {
        Some(old_pool.ok_or(biomcp_fetch::CacheError::Internal)?)
    } else {
        crate::publish::keep(
            snapshot::publish(
                cx,
                &key,
                &saved,
                saved.pages.iter().map(|p| p.0.len() as u32).sum(),
                std::slice::from_ref(&saved.source),
                old_pool,
                deadline,
            ),
            out,
        )?
    };
    present(q, out, saved, pool)
}
fn present(
    q: &AuthorSearch,
    out: &mut Outcome,
    saved: Saved,
    pool: Option<crate::store::Pool>,
) -> Result<(), RequestError> {
    let page = saved
        .pages
        .into_iter()
        .find(|p| p.0.offset() == q.offset())
        .ok_or(biomcp_fetch::CacheError::Internal)?
        .0;
    let next = page.next();
    let total = page.total();
    let mapped = page.headers();
    out.include_refusals(mapped.refusals);
    out.problems.extend(mapped.problems);
    let items = mapped.value;
    for row in &items {
        out.next.push(crate::next::Step::new(
            StepKind::Disambiguate,
            &[super::next::author(&row.id)],
            run(Request::GetAuthor(AuthorGet::one(row.id.clone()))),
        ));
    }
    if let (Some(offset), Some(pool)) = (next, pool.as_ref()) {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::SemanticScholar],
            run(Request::SearchAuthor(
                q.at_offset(offset).with_pool(pool.info.id.clone()),
            )),
        ));
    }
    out.page = total.map(|total| PageInfo {
        offset: q.offset(),
        returned: items.len() as u32,
        total,
        pool: pool.map(|p| p.info),
        truncated: None,
    });
    {
        let unavailable = saved.source.state == SourceState::Stale;
        crate::status::item(out, true, false, unavailable);
    };
    if saved.key_needed && saved.source.reason == Some(Reason::RateLimited) {
        super::key_step(out);
    }
    out.sources = vec![saved.source];
    out.body = (AuthorPage {
        items,
        matching_records: total,
        literature: None,
    })
    .with_query(q.clone());
    Ok(())
}

fn retained(
    cx: &Ctx<'_>,
    q: &AuthorSearch,
    key: &[u8],
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(Saved, crate::store::Pool)>, RequestError> {
    let (saved, mismatch) = if cx.fresh {
        (None, false)
    } else {
        snapshot::find::<Saved>(cx, key, q.pool(), deadline)?
    };
    let saved = saved.filter(|(saved, _)| {
        saved.pages.iter().any(|p| q.offset() == p.0.offset())
            || saved
                .pages
                .last()
                .is_some_and(|p| p.0.next() == Some(q.offset()))
    });
    if saved.is_none() && q.pool().is_some() {
        out.body = ReplyData::PoolError { mismatch };
        out.next.push(crate::next::Step::new(
            StepKind::Restart,
            &[Source::SemanticScholar],
            super::run(Request::SearchAuthor(q.restart())),
        ));
    }
    Ok(saved)
}
