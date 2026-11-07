use super::{empty, order, page};
use crate::{Ctx, Outcome, store::Store};
use biodata::Text;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::ReplyBody;
use biomcp_types::{ArticleEdges, ArticleItem, ArticleKey, ReplyData, Request, StepKind};
pub(super) fn run(cx: &Ctx<'_>, query: &ArticleEdges, references: bool) -> Outcome {
    let mut out = empty(
        (biomcp_types::ArticleEdgePage {
            items: Box::default(),
        })
        .with_query(query.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    if let Err(error) = execute(cx, query, references, &mut deadline, &mut out) {
        out.local = crate::status::local(error);
        crate::status::item(&mut out, false, true, false);
    }
    command.finish(deadline.remaining());
    out
}
fn request(query: ArticleEdges, references: bool) -> Request {
    if references {
        Request::ArticleReferences(query)
    } else {
        Request::ArticleCitations(query)
    }
}
fn restart(query: &ArticleEdges, references: bool, out: &mut Outcome, fresh: bool) {
    out.next.push(crate::next::Step::run(
        if fresh {
            StepKind::Fresh
        } else {
            StepKind::Restart
        },
        &[super::next::edge_source(query)],
        request(query.page(0, None), references),
        fresh,
    ));
}
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEdges,
    references: bool,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let mut store = Store::open(cx.fetch.cache_access()?, *deadline)?;
    let identity = serde_json::to_vec(&request(query.identity(), references))
        .map_err(|_| CacheError::Encoding)?;
    let keys = super::pool::key(cx, b"biomcp.article_edges/2\0", &identity)?;
    let key = keys.identity;
    let explicit = query.pool().map(Text::as_str);
    let pool = if cx.fresh && explicit.is_none() {
        None
    } else {
        store.find(&keys.current, explicit)?
    };
    let publish = pool.is_none() && explicit.is_none();
    let pool = match pool {
        Some(pool) if pool.key != key.as_slice() => {
            out.body = ReplyData::PoolError { mismatch: true };
            restart(query, references, out, false);
            return Ok(());
        }
        Some(pool) => pool,
        None if explicit.is_some() => {
            out.body = ReplyData::PoolError { mismatch: false };
            restart(query, references, out, false);
            return Ok(());
        }
        None => {
            let Some(items) = create(cx, query, references, deadline, out)? else {
                return Ok(());
            };
            out.dropped |= items
                .iter()
                .any(biomcp_adapters::article_rights::item_unchecked);
            store.set_deadline(*deadline);
            match crate::publish::keep(
                store
                    .publish(&key, &items, &out.sources, &out.problems, out.dropped)
                    .map_err(Into::into),
                out,
            )? {
                Some(pool) => pool,
                None => {
                    page::fetched(
                        cx,
                        deadline,
                        page::Cursor::Edges(query, references),
                        items,
                        out,
                    );
                    return Ok(());
                }
            }
        }
    };
    out.sources = pool.sources.clone();
    let result = super::pool::read(
        cx,
        page::Cursor::Edges(query, references),
        &mut store,
        &pool,
        out,
        keys.current.as_slice(),
        publish,
    );
    *deadline = store.deadline();
    result
}

fn create<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEdges,
    references: bool,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Option<Vec<ArticleItem>>, RequestError> {
    if query.source() == biomcp_types::EdgeSource::SemanticScholar {
        return super::semantic_edges::create(cx, query, references, deadline, out);
    }
    let Some(pmid) = query.pmid() else {
        return Ok(None);
    };
    let parent = if references {
        order::fetch_references(cx, deadline, &[pmid])
    } else {
        order::fetch(cx, deadline, &[pmid])
    };
    crate::status::append(&mut out.problems, parent.problems);
    out.sources.push(parent.source);
    if let Some(error) = parent.local {
        return Err(error);
    }
    if parent.error.is_some() {
        crate::status::item(out, false, true, false);
        restart(query, references, out, true);
        return Ok(None);
    }
    let ids = parent
        .rows
        .into_iter()
        .find(|row| row.pmid == pmid)
        .and_then(|row| {
            if references {
                row.references
            } else {
                row.citing
            }
        });
    let Some(ids) = ids else {
        crate::status::item(out, false, true, false);
        if let Some(source) = out.sources.last_mut() {
            source.set_status(biomcp_types::SourceStatus::Failed(Some(
                biomcp_types::Reason::BadReply,
            )));
        }
        restart(query, references, out, true);
        return Ok(None);
    };
    if order::refuse(ids.len(), out) {
        return Ok(None);
    }
    let mut items: Vec<_> = ids
        .into_iter()
        .map(|id| ArticleItem {
            text_withheld: None,
            context_license: None,
            key: Some(ArticleKey::Pubmed(id)),
            pmid: Some(id),
            header: None,
            cited_by: None,
            intents: None,
            influential: None,
            contexts: None,
            found_by: Default::default(),
            sentence: None,
        })
        .collect();
    if !order::apply(cx, deadline, &mut items, out)? {
        restart(query, references, out, true);
        return Ok(None);
    }
    Ok(Some(items))
}
