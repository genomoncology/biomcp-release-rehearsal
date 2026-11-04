//! One source-order navigation policy for an association page.
use biomcp_types::Source;
use biomcp_types::{GwasPageQuery, GwasSearch, Request, StepKind};
type Steps = Vec<crate::next::Step>;
pub(super) fn search(
    query: &GwasSearch,
    page: Option<&biomcp_types::GwasPage>,
    state: biomcp_types::SourceState,
    fresh: bool,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Steps {
    let mut next = Vec::new();
    let rows = page.map(|page| page.items.as_ref()).unwrap_or_default();
    if let Some(GwasPageQuery::Search(query)) = page.and_then(|page| page.continuation.as_ref()) {
        push(
            &mut next,
            StepKind::Page,
            Request::SearchGwas(query.clone()),
            fresh && query.pool().is_none(),
            Source::Gwas,
        );
    }
    let before = next.len();
    let mut seen = std::collections::BTreeSet::new();
    for row in rows {
        let mut requests = Vec::new();
        for allele in &row.alleles {
            if let Some(get) = crate::next::checked(
                allele
                    .rs_id
                    .as_str()
                    .parse()
                    .map_err(|_| biomcp_types::QueryInvalid::new(&["input"]))
                    .and_then(|input| {
                        biomcp_types::VariantGet::new(
                            input,
                            biomcp_types::VariantResolver::Car,
                            None,
                        )
                    }),
                Source::Gwas,
                problems,
            ) {
                requests.push((Request::GetVariant(get), Source::MyVariant));
            }
        }
        if let Some(publication) = &row.publication
            && let Some(get) = crate::next::checked(
                publication
                    .as_str()
                    .parse()
                    .map_err(|_| biomcp_types::QueryInvalid::new(&["input"]))
                    .and_then(|input| biomcp_types::ArticleGet::new(vec![input].into())),
                Source::Gwas,
                problems,
            )
        {
            requests.push((Request::GetArticle(get), Source::PubMed));
        }
        for (request, target) in requests {
            let key = serde_json::to_string(&request).ok();
            if next.len() - before < 3 && key.is_some_and(|key| seen.insert(key)) {
                push(&mut next, StepKind::Pivot, request, fresh, target);
            }
        }
    }
    if matches!(
        state,
        biomcp_types::SourceState::Failed | biomcp_types::SourceState::Stale
    ) {
        push(
            &mut next,
            StepKind::Fresh,
            Request::SearchGwas(query.clone()),
            true,
            Source::Gwas,
        );
    }
    next
}
fn push(next: &mut Steps, kind: StepKind, request: Request, fresh: bool, source: Source) {
    next.push(crate::next::Step::run(kind, &[source], request, fresh));
}
