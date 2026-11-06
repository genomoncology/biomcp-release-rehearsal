//! Citation author discovery does not infer identities from matching names.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::pubmed;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    AuthorLiterature, AuthorPage, AuthorPublication, AuthorSearch, AuthorSearchSource, PageInfo,
    Paged, ReplyBody, Request, Source, StepKind,
};

pub(super) fn search(cx: &Ctx<'_>, q: &AuthorSearch) -> Outcome {
    if q.pool().is_some() {
        let mut out = status::empty(biomcp_types::ReplyData::PoolError { mismatch: true });
        out.next.push(crate::next::Step::new(
            StepKind::Restart,
            &[Source::PubMed],
            super::run(Request::SearchAuthor(q.restart())),
        ));
        return out;
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = status::empty(AuthorPage::into_reply(q.clone(), None));
    if let Err(error) = execute(cx, q, deadline, &mut out) {
        out.local = status::local(error);
    }
    command.finish(deadline.remaining());
    out
}

fn execute(
    cx: &Ctx<'_>,
    q: &AuthorSearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&pubmed::SPEC, deadline).fresh(cx.fresh);
    let mut notices = Vec::new();
    let mut count = None;
    let result = (|| {
        let found = pubmed::search_ids(
            &leg,
            q.query(),
            biomcp_types::ArticleSort::Rank,
            cx.ncbi_email.as_ref(),
        )?;
        out.problems.extend(found.problems);
        notices = found.notices;
        count = Some(found.count);
        let ids: Vec<_> = found
            .ids
            .into_iter()
            .skip(q.offset() as usize)
            .take(q.limit() as usize)
            .collect();
        let mapped = if ids.is_empty() {
            None
        } else {
            Some(pubmed::author_summaries(
                &leg,
                &ids,
                cx.ncbi_email.as_ref(),
            )?)
        };
        let papers = if let Some(mapped) = mapped {
            out.include_refusals(mapped.refusals);
            out.problems.extend(mapped.problems);
            mapped.value
        } else {
            Vec::new()
        };
        present(q, out, papers, found.count, ids.len() as u32)
    })();
    let error = result.as_ref().err().copied();
    let mut source = status::source(Source::PubMed, leg.finish(), error);
    source.term = match q.query().parse() {
        Ok(term) => Some(term),
        Err(_) => {
            out.problems.push(biomcp_types::Problem {
                source: Source::PubMed,
                record: None,
                field: "term".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
            None
        }
    };
    source.count = count;
    source.notices = notices;
    out.sources.push(source);
    // Offset stays: no pool ties this page, and a step naming a disabled source is dropped.
    super::retry(out, Request::SearchAuthor(q.clone()));
    // The source can explain PubMed's query interpretation without claiming identity resolution.
    if q.source() == AuthorSearchSource::Auto && cx.fetch.disabled(Source::SemanticScholar) {
        out.sources.push(biomcp_adapters::registry::outcome(
            Source::SemanticScholar,
            biomcp_types::SourceStatus::Skipped(Some(biomcp_types::Reason::Disabled)),
        ));
    }
    if result.is_ok() {
        status::item(out, true, false, false);
    }
    result
}

fn present(
    q: &AuthorSearch,
    out: &mut Outcome,
    papers: Vec<AuthorPublication>,
    count: u64,
    consumed: u32,
) -> Result<(), RequestError> {
    let returned = papers.len() as u32;
    if q.offset().saturating_add(consumed) < count.min(9999) as u32 {
        let next = q.with_source(AuthorSearchSource::Pubmed);
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[Source::PubMed],
            super::run(Request::SearchAuthor(
                next.at_offset(q.offset() + q.limit()),
            )),
        ));
    }
    out.page = Some(PageInfo {
        offset: q.offset(),
        returned,
        total: count,
        pool: None,
        truncated: (count > 9999).then_some(true),
    });
    out.body = AuthorPage {
        items: Box::default(),
        matching_records: None,
        literature: Some(AuthorLiterature {
            identity_resolution: concat!(
                "Unresolved: these are author names on individual papers, not verified researcher identities. ",
                "Equal names may refer to different people; names are not merged. ",
                "Affiliations, author-specific publication totals, citation metrics and topic expertise are not established. ",
                "PubMed matches papers, not people; only the first 9999 matching papers can be paged."
            ).parse().map_err(|_| biomcp_fetch::CacheError::Encoding)?,
            papers: papers.into(),
        }),
    }.with_query(q.clone());
    Ok(())
}
