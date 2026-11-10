//! Reuse complete source-stated names, then apply the whole native selector bound.
use super::Saved;
use crate::{Ctx, drug::search_names, status};
use biodata::Text;
use biomcp_fetch::Deadline;
use biomcp_types::{DrugSearch, ReplyData, SourceNotice};
pub(super) fn resolve<'a>(cx: &Ctx<'a>, query: &DrugSearch, deadline: &mut Deadline<'a>) -> Saved {
    let mut out = status::empty(ReplyData::PoolError { mismatch: false });
    let mut names: Vec<Text> = query.query().cloned().into_iter().collect();
    let mut incomplete = false;
    let mut omitted = 0;
    search_names::bounded(deadline, |lookup| {
        if let Some(input) = query.query() {
            let found = search_names::resolve(cx, input, &mut out, lookup, |_| true);
            incomplete = found.failed;
            for name in found.names {
                if names
                    .iter()
                    .any(|prior| prior.as_str().eq_ignore_ascii_case(name.as_str()))
                {
                    continue;
                }
                let mut candidate = names.clone();
                candidate.push(name);
                if query.label_name_selector(&candidate, true).is_ok() {
                    names = candidate;
                } else {
                    omitted += 1;
                }
            }
        }
    });
    for source in &mut out.sources {
        for notice in &mut source.notices {
            if let SourceNotice::DrugNameSearch {
                names: searched,
                message,
                ..
            } = notice
            {
                *searched = searched
                    .iter()
                    .filter(|name| names.contains(name))
                    .cloned()
                    .collect();
                if omitted != 0 {
                    *message = format!(
                        "{message} Additional names exceeded the whole native query bound."
                    )
                    .into();
                }
            }
        }
    }
    Saved {
        names,
        sources: out.sources,
        problems: out.problems,
        incomplete,
        omitted,
        tier: None,
    }
}
