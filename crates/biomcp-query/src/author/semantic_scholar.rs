use super::{empty, recovery, run};
use crate::{Ctx, Outcome, source_snapshot as snapshot, status};
use biodata::{AuthorPaperHeader, SemanticScholarAuthorId};
use biomcp_adapters::semantic_scholar::{self, author};
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    ArticleGet, ArticleInput, AuthorPaperDetails, AuthorPapers, AuthorPapersQuery, PageInfo,
    Reason, ReplyData, Request, Source, SourceState, StepKind,
};
/// Read one exact author's native paper page and retain checked continuation state.
pub(super) fn papers(cx: &Ctx<'_>, q: &AuthorPapersQuery, id: &SemanticScholarAuthorId) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = empty(body(q, Box::default()));
    if let Err(error) = execute(cx, q, id, deadline, &mut out) {
        out.local = status::local(error);
        if matches!(error, RequestError::Source(e) if e.reason == Reason::NotFound) {
            crate::status::item(&mut out, false, false, false);
        }
        if out.sources.is_empty() {
            out.sources.push(biomcp_adapters::registry::outcome(
                Source::SemanticScholar,
                biomcp_types::SourceStatus::Failed(None),
            ));
        }
    }
    recovery(cx, &mut out, Request::AuthorPapers(q.restart()));
    command.finish(deadline.remaining());
    out
}
fn execute(
    cx: &Ctx<'_>,
    q: &AuthorPapersQuery,
    id: &SemanticScholarAuthorId,
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
    let key = snapshot::key("author-papers", &(q.id(), q.section(), q.limit()))?;
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
    let cached = saved.pages.iter().any(|p| p.offset == q.offset());
    if !cached {
        let leg = cx
            .fetch
            .leg(&semantic_scholar::SPEC, deadline)
            .fresh(cx.fresh);
        let result = author::papers(&leg, id, q);
        let error = result.as_ref().err().copied();
        let source = status::source(Source::SemanticScholar, leg.finish(), error);
        out.sources.push(source.clone());
        saved.key_needed |= source.reason == Some(Reason::RateLimited)
            && !cx
                .fetch
                .has_key(biomcp_fetch::SecretName::SemanticScholarApiKey);
        saved.pages.push(result?);
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
                saved.pages.iter().map(|p| p.data.len() as u32).sum(),
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
    q: &AuthorPapersQuery,
    out: &mut Outcome,
    saved: super::NativePages<author::PapersPage>,
    pool: Option<crate::store::Pool>,
) -> Result<(), RequestError> {
    let page = saved
        .pages
        .into_iter()
        .find(|p| p.offset == q.offset())
        .ok_or(biomcp_fetch::CacheError::Internal)?;
    out.include_refusals(page.refusals);
    out.problems.extend(page.problems);
    let items: Box<[_]> = page
        .data
        .into_iter()
        .map(|r| r.detail(&mut out.problems))
        .collect::<Result<_, _>>()
        .map_err(|_| SourceError {
            source: Source::SemanticScholar,
            reason: Reason::BadReply,
            status: None,
            tls_cause: None,
            retry_after: None,
        })?;
    for row in &items {
        pivot(&row.header, out);
    }
    if let (Some(offset), Some(pool)) = (page.next, pool.as_ref()) {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::SemanticScholar],
            run(Request::AuthorPapers(
                q.at_offset(offset).with_pool(pool.info.id.clone()),
            )),
        ));
    }
    out.page = page.total.map(|total| PageInfo {
        offset: q.offset(),
        returned: items.len() as u32,
        total,
        pool: pool.map(|p| p.info),
        truncated: None,
    });
    {
        let (found, identity_failed, unavailable) = if saved.source.state == SourceState::Stale {
            (true, false, true)
        } else {
            (true, false, false)
        };
        crate::status::item(out, found, identity_failed, unavailable);
    };
    if saved.key_needed && saved.source.reason == Some(Reason::RateLimited) {
        super::key_step(out);
    }
    out.sources = vec![saved.source];
    out.body = body(q, items);
    Ok(())
}
fn body(q: &AuthorPapersQuery, items: Box<[biodata::AuthorPaperDetail]>) -> ReplyData {
    if q.section().is_some() {
        (AuthorPaperDetails {
            author: q.id().clone(),
            items,
        })
        .with_query(q.clone())
    } else {
        (AuthorPapers {
            author: q.id().clone(),
            items: items.into_iter().map(|r| r.header).collect(),
        })
        .with_query(q.clone())
    }
}
fn pivot(row: &AuthorPaperHeader, out: &mut Outcome) {
    let input = row
        .pmid
        .map(ArticleInput::Pmid)
        .or_else(|| row.doi.clone().map(ArticleInput::Doi));
    if let Some(input) = input
        && let Some(q) = crate::next::checked(
            ArticleGet::with_sections(vec![input].into(), Box::default()),
            biomcp_types::Source::SemanticScholar,
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::new(
            StepKind::Pivot,
            &super::super::article::next::get_sources(&q),
            run(Request::GetArticle(q)),
        ));
    }
}

fn retained(
    cx: &Ctx<'_>,
    q: &AuthorPapersQuery,
    key: &[u8],
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<(super::NativePages<author::PapersPage>, crate::store::Pool)>, RequestError> {
    let (saved, mismatch) = if cx.fresh {
        (None, false)
    } else {
        snapshot::find::<super::NativePages<author::PapersPage>>(cx, key, q.pool(), deadline)?
    };
    let saved = saved.filter(|(saved, _)| {
        saved.pages.iter().any(|p| q.offset() == p.offset)
            || saved
                .pages
                .last()
                .is_some_and(|p| p.next == Some(q.offset()))
    });
    if saved.is_none() && q.pool().is_some() {
        out.body = ReplyData::PoolError { mismatch };
        out.next.push(crate::next::Step::new(
            StepKind::Restart,
            &[super::next::author(q.id())],
            super::run(Request::AuthorPapers(q.restart())),
        ));
    }
    Ok(saved)
}
