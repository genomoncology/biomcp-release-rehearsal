use super::consume;
use super::sources;
use crate::{
    Ctx, Outcome, status,
    store::{Pool, Store, StoredItem},
};
use biodata::Text;
use biomcp_adapters::pubmed;
use biomcp_fetch::RequestError;
use biomcp_types::ReplyBody;
use biomcp_types::{
    ArticleEdgePage, ArticleEdges, ArticlePage, ArticleSearch, ArticleSort, PageInfo, ReplyData,
    Request, Source, StepKind,
};
#[derive(Clone, Copy)]
pub(super) enum Cursor<'a> {
    Search(&'a ArticleSearch),
    Recommend(&'a biomcp_types::ArticleRecommend),
    Edges(&'a ArticleEdges, bool),
}
impl Cursor<'_> {
    fn sources(self) -> Vec<Source> {
        match self {
            Self::Search(query) => super::next::search_sources(query),
            Self::Recommend(_) => vec![Source::SemanticScholar],
            Self::Edges(query, _) => vec![super::next::edge_source(query)],
        }
    }

    fn offset(self) -> u32 {
        match self {
            Self::Search(q) => q.offset(),
            Self::Recommend(q) => q.offset(),
            Self::Edges(q, _) => q.offset(),
        }
    }
    fn limit(self) -> u32 {
        match self {
            Self::Search(q) => q.limit(),
            Self::Recommend(q) => q.limit(),
            Self::Edges(q, _) => q.limit(),
        }
    }
    fn request(self, offset: u32, pool: Option<Text>) -> Request {
        match self {
            Self::Search(q) => Request::SearchArticle(q.page(offset, pool)),
            Self::Recommend(q) => Request::ArticleRecommend(q.page(offset, pool)),
            Self::Edges(q, true) => Request::ArticleReferences(q.page(offset, pool)),
            Self::Edges(q, false) => Request::ArticleCitations(q.page(offset, pool)),
        }
    }
    fn retry(self, pool: Option<Text>, out: &mut Outcome) {
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &self.sources(),
            self.request(self.offset(), pool),
            true,
        ));
    }
}
pub(super) fn read<'a>(
    cx: &Ctx<'a>,
    query: Cursor<'_>,
    store: &mut Store<'a>,
    pool: &Pool,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if pool
        .sources
        .iter()
        .any(|s| cx.fetch.disabled(s.source) && s.reason != Some(biomcp_types::Reason::Disabled))
    {
        out.sources = withheld_sources(cx, pool);
        crate::status::item(out, false, true, false);
        return Ok(());
    }
    out.dropped |= pool.dropped;
    status::append(&mut out.problems, pool.problems.clone());
    let id = pool.info.id.as_str();
    let mut rows = store.page(id, query.offset(), query.limit())?;
    let missing: Vec<_> = rows
        .iter()
        .filter(|r| r.item.header.is_none() && (!r.checked || cx.fresh))
        .filter_map(|r| r.item.pmid)
        .collect();
    if !missing.is_empty() {
        let deadline = store.deadline();
        let leg = cx.fetch.leg(&pubmed::SPEC, deadline).fresh(cx.fresh);
        let started = leg.remaining();
        let result = pubmed::summaries(&leg, &missing, cx.ncbi_email.as_ref());
        store.set_deadline(consume(deadline, started, leg.remaining()));
        match result {
            Ok(mapped) => {
                out.include_refusals(mapped.refusals);
                let source = status::source(Source::PubMed, leg.finish(), None);
                if let Some(current) = out.sources.iter_mut().find(|s| s.source == Source::PubMed) {
                    status::retain_source_state(current, &source);
                } else {
                    out.sources.push(source.clone());
                }
                store.hydrate(
                    id,
                    &missing,
                    &mapped.value,
                    &mapped.problems,
                    Some(&source),
                    out.dropped,
                )?;
                rows = store.page(id, query.offset(), query.limit())?;
            }
            Err(error) => {
                let mut source = status::source(Source::PubMed, leg.finish(), Some(error));
                if let Some(original) = pool.sources.iter().find(|s| s.source == Source::PubMed) {
                    source.term = original.term.clone();
                    source.count = original.count;
                    source.retrieved_count = original.retrieved_count;
                    source.notices.extend(original.notices.clone());
                }
                out.sources.retain(|s| s.source != Source::PubMed);
                out.sources.push(source);
                if !matches!(error, RequestError::Source(e) if e.reason == biomcp_types::Reason::Disabled)
                {
                    crate::status::item(out, false, true, false);
                    query.retry(Some(pool.info.id.clone()), out);
                    return Err(error);
                }
            }
        }
    }
    filter_stored_text(cx, store, id, &mut rows, out)?;
    finish(query, pool, rows, out);
    Ok(())
}
/// Recheck and rewrite retained article text before returning a stored page.
fn filter_stored_text<'a>(
    cx: &Ctx<'a>,
    store: &mut Store<'a>,
    id: &str,
    rows: &mut [StoredItem],
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if cx.fetch.disabled_use(biomcp_types::Use::PerRecord) {
        let leg = cx.fetch.leg(&pubmed::SPEC, store.deadline());
        let started = leg.remaining();
        let mut items: Vec<_> = rows.iter().map(|row| row.item.clone()).collect();
        out.dropped |= biomcp_adapters::article_rights::items(&leg, &mut items);
        let mut changed = Vec::new();
        for (row, item) in rows.iter_mut().zip(items) {
            // An unfinished lookup settles nothing, so its row stays as stored.
            if row.item != item && !biomcp_adapters::article_rights::item_unchecked(&item) {
                changed.push(item.clone());
            }
            row.item = item;
        }
        store.set_deadline(consume(store.deadline(), started, leg.remaining()));
        if !changed.is_empty() {
            store.rewrite_article_text(id, &changed)?;
        }
    }
    Ok(())
}

fn withheld_sources(cx: &Ctx<'_>, pool: &Pool) -> Vec<biomcp_types::SourceOutcome> {
    // Expansion terms and provider warnings can repeat names from a disabled component.
    let expanded = pool
        .sources
        .iter()
        .flat_map(|source| &source.notices)
        .any(|notice| matches!(notice, biomcp_types::SourceNotice::DrugNameSearch { .. }));
    pool.sources
        .iter()
        .cloned()
        .map(|mut s| {
            if expanded {
                s.term = None;
                s.notices.clear();
            }
            if cx.fetch.disabled(s.source) {
                s.set_status(biomcp_types::SourceStatus::Skipped(Some(
                    biomcp_types::Reason::Disabled,
                )));
                s.count = None;
                s.retrieved_count = None;
            }
            s
        })
        .collect()
}
fn finish(query: Cursor<'_>, pool: &Pool, rows: Vec<StoredItem>, out: &mut Outcome) {
    for source in rows.iter().filter_map(|r| r.outcome.as_ref()) {
        if let Some(original) = out.sources.iter_mut().find(|s| s.source == source.source) {
            status::retain_source_state(original, source);
        } else {
            out.sources.push(source.clone());
        }
    }
    let stale = out
        .sources
        .iter()
        .any(|s| s.state == biomcp_types::SourceState::Stale);
    let failed = sources::failed(&out.sources);
    let partial = stale || failed || rows.iter().any(|r| r.item.header.is_none());
    for row in &rows {
        status::append(&mut out.problems, row.problems.clone());
    }
    {
        let unavailable = partial;
        crate::status::item(out, true, false, unavailable);
    };
    let missing_external = rows
        .iter()
        .any(|r| r.item.pmid.is_none() && r.item.header.is_none());
    let returned = u32::try_from(rows.len()).unwrap_or(u32::MAX);
    let total = u64::from(pool.info.size);
    out.page = Some(PageInfo {
        offset: query.offset(),
        returned,
        total,
        pool: Some(pool.info.clone()),
        truncated: Some(pool.sources.iter().any(|s| {
            s.count
                .zip(s.retrieved_count)
                .is_some_and(|(count, fetched)| count > u64::from(fetched))
                || s.notices
                    .iter()
                    .any(|n| matches!(n, biomcp_types::SourceNotice::GraphTruncated { .. }))
        })),
    });
    let items = rows.into_iter().map(|r| r.item).collect();
    out.body = match query {
        Cursor::Search(q) => (ArticlePage {
            items,
            candidates: Vec::new(),
        })
        .with_query(q.clone()),
        Cursor::Edges(q, _) => (ArticleEdgePage { items }).with_query(q.clone()),
        Cursor::Recommend(q) => (super::recommend::body(items, total == 0)).with_query(q.clone()),
    };
    actions(query, pool, returned, total, out);
    if let Cursor::Search(search) = query {
        super::next::search(search, total, partial, out);
    }
    if partial {
        query.retry(
            if stale || failed || missing_external {
                None
            } else {
                Some(pool.info.id.clone())
            },
            out,
        );
    }
    if let Cursor::Search(search) = query
        && search.sort() != ArticleSort::Citations
    {
        out.next.push(crate::next::Step::run(
            StepKind::Sort,
            &super::next::search_sources(search),
            Request::SearchArticle(search.ordered(ArticleSort::Citations)),
            false,
        ));
    }
}

fn actions(query: Cursor<'_>, pool: &Pool, returned: u32, total: u64, out: &mut Outcome) {
    if let Some(offset) = query
        .offset()
        .checked_add(returned)
        .filter(|offset| u64::from(*offset) < total && returned > 0)
    {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &query.sources(),
            query.request(offset, Some(pool.info.id.clone())),
            false,
        ));
    }
    let items = match &out.body {
        ReplyData::ArticlePage {
            body: Some(page), ..
        } => page.items.as_ref(),
        ReplyData::ArticleEdgePage {
            body: Some(page), ..
        } => page.items.as_ref(),
        _ => &[],
    };
    if let Cursor::Edges(parent, references) = query
        && let Some(item) = items
            .iter()
            .find(|item| item.contexts.as_ref().is_some_and(|c| c.total > 1))
    {
        let input =
            item.pmid
                .map(biomcp_types::ArticleInput::Pmid)
                .or_else(|| match item.key.as_ref() {
                    Some(biomcp_types::ArticleKey::Doi(id)) => {
                        Some(biomcp_types::ArticleInput::Doi(id.clone()))
                    }
                    Some(biomcp_types::ArticleKey::Pmc(id)) => {
                        Some(biomcp_types::ArticleInput::Pmcid(*id))
                    }
                    _ => None,
                });
        if let Some(input) = input {
            let (citing, cited) = if references {
                (parent.input().clone(), input)
            } else {
                (input, parent.input().clone())
            };
            let evidence = biomcp_types::ArticleEvidence::new(citing, cited);
            out.next.push(crate::next::Step::run(
                StepKind::Disambiguate,
                &super::next::evidence_sources(&evidence),
                Request::ArticleEvidence(evidence),
                false,
            ));
        }
    }
    if let Some(first) = items.first()
        && let Some(id) = first.pmid
        && let Some(query) = crate::next::checked(
            biomcp_types::ArticleGet::new(vec![biomcp_types::ArticleInput::Pmid(id)].into()),
            biomcp_types::Source::PubMed,
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::run(
            StepKind::Disambiguate,
            &super::next::get_sources(&query),
            Request::GetArticle(query),
            false,
        ));
    }
    super::next::authors(out);
}

/// Return the checked live window when no retained collection could be written.
pub(super) fn fetched(query: Cursor<'_>, items: Vec<biomcp_types::ArticleItem>, out: &mut Outcome) {
    let total = items.len() as u64;
    let items: Box<[_]> = items
        .into_iter()
        .skip(query.offset() as usize)
        .take(query.limit() as usize)
        .collect();
    out.page = Some(PageInfo {
        offset: query.offset(),
        returned: items.len() as u32,
        total,
        pool: None,
        truncated: None,
    });
    out.body = match query {
        Cursor::Search(q) => ArticlePage {
            items,
            candidates: Vec::new(),
        }
        .with_query(q.clone()),
        Cursor::Edges(q, _) => ArticleEdgePage { items }.with_query(q.clone()),
        Cursor::Recommend(q) => super::recommend::body(items, total == 0).with_query(q.clone()),
    };
    status::item(out, true, false, false);
}
