//! Substitute source-stated drug names without changing the caller's other filters.
use crate::{Ctx, Outcome, drug::search_names};
use biodata::Text;
use biomcp_types::{SourceNotice, TrialSearch};

/// The registry request, and which typed terms it replaced.
pub(super) struct Expanded {
    pub query: TrialSearch,
    pub inputs: Vec<Text>,
    pub failed: bool,
}

/// A page token already belongs to one registry query, so continuing never looks names up.
pub(super) fn expand<'a>(
    cx: &Ctx<'a>,
    query: &TrialSearch,
    out: &mut Outcome,
    deadline: &mut biomcp_fetch::Deadline<'a>,
) -> Expanded {
    let mut expanded = Expanded {
        query: query.clone(),
        inputs: Vec::new(),
        failed: false,
    };
    if query.exact_term()
        || query.next_page().is_some()
        || cx.fetch.disabled(query.source().source())
    {
        return expanded;
    }
    search_names::bounded(deadline, |lookup| {
        let mut term = |input: &Text| {
            let resolved =
                search_names::resolve(cx, input, out, lookup, biomcp_adapters::ctgov::name_fits);
            expanded.failed |= resolved.failed;
            if resolved.names.len() < 2 {
                return input.clone();
            }
            expanded.inputs.push(input.clone());
            search_names::alternatives(&resolved.names, input)
        };
        let text = query.query().map(&mut term);
        let interventions = query.intervention().iter().map(&mut term).collect();
        let keywords = query.keyword().iter().map(&mut term).collect();
        expanded.query = query.with_drug_terms(text, interventions, keywords);
    });
    expanded
}

/// NCI search takes no name group, so its reply says the terms stayed as typed.
pub(super) fn literal_notices(query: &TrialSearch) -> Vec<SourceNotice> {
    if query.exact_term() {
        return Vec::new();
    }
    query
        .query()
        .into_iter()
        .chain(query.intervention())
        .chain(query.keyword())
        .map(|input| SourceNotice::DrugNameSearch {
            input: input.clone(),
            names: vec![input.clone()].into(),
            message: "NCI search does not expand drug names; searched the typed term.".into(),
        })
        .collect()
}
