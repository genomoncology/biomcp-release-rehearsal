//! Source-native recommendation windows reuse the ordinary checked article pool.
use crate::{Action, Ctx, Outcome, status, store::Store};
use biodata::Text;
use biomcp_adapters::semantic_scholar;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::ReplyBody;
use biomcp_types::{
    ArticleRecommend, ArticleRecommendations, EvidenceNotice, ReplyData, Request, Source, Status,
    StepKind,
};
use sha2::{Digest, Sha256};
/// Read one recommendation window without local topic filtering or ranking.
pub(crate) fn run(cx: &Ctx<'_>, query: &ArticleRecommend) -> Outcome {
    let mut out = super::empty((body(Box::default(), false)).with_query(query.clone()));
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    if let Err(error) = execute(cx, query, &mut deadline, &mut out) {
        out.local = status::local(error);
        crate::status::item(&mut out, false, true, false);
    }
    command.finish(deadline.remaining());
    if out.status() == Status::Failed {
        if out
            .sources
            .iter()
            .any(|s| s.reason == Some(biomcp_types::Reason::RateLimited))
        {
            out.next.push(crate::next::Step::new(
                StepKind::Key,
                &[biomcp_types::Source::SemanticScholar],
                Action::KeyHelp(biomcp_fetch::SecretName::SemanticScholarApiKey),
            ));
        }
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &[biomcp_types::Source::SemanticScholar],
            Request::ArticleRecommend(query.page(0, None)),
            true,
        ));
    }
    out
}
pub(super) fn body(items: Box<[biomcp_types::ArticleItem]>, empty: bool) -> ArticleRecommendations {
    ArticleRecommendations {
        items,
        scope: "Semantic Scholar recommends from its pool of recent papers".into(),
        notice: empty.then_some(EvidenceNotice::NoRecommendations),
    }
}
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticleRecommend,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let mut store = Store::open(cx.fetch.cache_access()?, *deadline)?;
    let identity = serde_json::to_vec(&query.identity()).map_err(|_| CacheError::Encoding)?;
    let key = Sha256::digest([b"biomcp.article_recommend/1\0".as_slice(), &identity].concat());
    let explicit = query.pool().map(Text::as_str);
    let old = if cx.fresh && explicit.is_none() {
        None
    } else {
        store.find(&key, explicit)?
    };
    let pool = match old {
        Some(pool) if pool.key != key.as_slice() => {
            out.body = ReplyData::PoolError { mismatch: true };
            restart(query, out);
            return Ok(());
        }
        Some(pool) => pool,
        None if explicit.is_some() => {
            out.body = ReplyData::PoolError { mismatch: false };
            restart(query, out);
            return Ok(());
        }
        None => {
            let items = create(cx, query, deadline, out)?;
            store.set_deadline(*deadline);
            match crate::publish::keep(
                store
                    .publish(&key, &items, &out.sources, &out.problems, out.dropped)
                    .map_err(Into::into),
                out,
            )? {
                Some(pool) => pool,
                None => {
                    super::page::fetched(super::page::Cursor::Recommend(query), items, out);
                    return Ok(());
                }
            }
        }
    };
    out.sources = pool.sources.clone();
    let result = super::page::read(
        cx,
        super::page::Cursor::Recommend(query),
        &mut store,
        &pool,
        out,
    );
    *deadline = store.deadline();
    result
}

fn create<'a>(
    cx: &Ctx<'a>,
    query: &ArticleRecommend,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Vec<biomcp_types::ArticleItem>, RequestError> {
    let leg = cx
        .fetch
        .leg(&semantic_scholar::RECOMMEND_SPEC, *deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let reply =
        semantic_scholar::recommendations(&leg, query.seeds(), query.negative(), query.limit());
    *deadline = super::consume(*deadline, started, leg.remaining());
    out.sources.push(status::source(
        Source::SemanticScholar,
        leg.finish(),
        reply.as_ref().err().copied(),
    ));
    let reply = reply?;
    out.include_refusals(reply.refusals);
    out.problems.extend(reply.problems);
    let dois: Vec<_> = reply
        .recommended_papers
        .iter()
        .map(|p| p.external_ids.as_ref().and_then(|ids| ids.doi.clone()))
        .collect();
    let mut items: Vec<_> = reply
        .recommended_papers
        .into_iter()
        .map(|paper| paper.item(&mut out.problems))
        .collect();
    if items.len() > query.limit() as usize {
        return Err(biomcp_fetch::SourceError {
            source: Source::SemanticScholar,
            reason: biomcp_types::Reason::BadReply,
            status: Some(200),
            tls_cause: None,
            retry_after: None,
        }
        .into());
    }
    if items
        .iter()
        .zip(&dois)
        .any(|(item, doi)| item.pmid.is_none() && doi.is_some())
    {
        let joined = super::semantic_scholar::resolve_dois_with(cx, &mut items, deadline, &dois);
        out.sources.push(joined.source);
        status::append(&mut out.problems, joined.problems);
        if let Some(error) = joined.error.and_then(status::local) {
            return Err(error);
        }
    }
    for item in &mut items {
        if item.pmid.is_some() && item.header.as_ref().is_some_and(|h| h.types_unknown) {
            item.header = None
        }
    }
    Ok(items)
}

fn restart(query: &ArticleRecommend, out: &mut Outcome) {
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[biomcp_types::Source::SemanticScholar],
        Request::ArticleRecommend(query.page(0, None)),
        false,
    ));
}
