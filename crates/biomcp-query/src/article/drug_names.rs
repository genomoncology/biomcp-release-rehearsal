//! Keep explicit snapshots stable while replacing legacy implicit literal-name pools.
use crate::{Ctx, Outcome, drug::search_names::Resolved, store::Pool};
use biomcp_types::{ArticleSearch, SourceNotice};

pub(super) fn reuse<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    stored: Option<Pool>,
    out: &mut Outcome,
    deadline: &mut biomcp_fetch::Deadline<'a>,
) -> (Option<Pool>, Option<Resolved>) {
    if query.pool().is_some()
        || query.exact_term()
        || stored.as_ref().is_none_or(|pool| {
            pool.sources
                .iter()
                .flat_map(|source| &source.notices)
                .any(|notice| matches!(notice, SourceNotice::DrugNameSearch { .. }))
        })
    {
        return (stored, None);
    }
    let resolved = crate::drug::search_names::article(cx, query, out, deadline);
    let stored = stored.filter(|_| resolved.names.is_empty() && !resolved.failed);
    (stored, Some(resolved))
}
