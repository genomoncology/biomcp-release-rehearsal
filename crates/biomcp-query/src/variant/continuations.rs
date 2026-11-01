//! Spell each provider's typed continuation without changing source identity or page selectors.
use crate::Action;
use biomcp_types::Paged;

use biomcp_types::{
    Request, Section, Source, StepKind, VariantGet, VariantHeader, VariantResolution,
};
pub(super) fn collect(
    fetch: &biomcp_fetch::Fetch,
    candidates: &[VariantHeader],
    next: &mut Vec<crate::next::Step>,
) {
    for candidate in candidates {
        let Some(sections) = candidate.sections.as_deref() else {
            continue;
        };
        for (source, section) in [
            (Source::ClinVar, &sections.clinvar),
            (Source::Erepo, &sections.erepo),
        ] {
            let Some(section) = section else { continue };
            if let Section::Data { data, .. } = section
                && let Some(query) = &data.next
            {
                push(
                    next,
                    StepKind::Page,
                    &[source],
                    Request::VariantAssertions(query.clone()),
                );
            }
        }
        if let Some(query) =
            data(sections.gwas.as_ref()).and_then(|gwas| gwas.continuation.as_ref())
        {
            push(
                next,
                StepKind::Page,
                &super::next_sources(fetch, query),
                Request::GetVariant(query.clone()),
            );
        }
        for query in [
            data(sections.civic.as_ref()).and_then(|d| d.next.as_ref()),
            data(sections.hotspots.as_ref()).and_then(|d| d.next.as_ref()),
        ]
        .into_iter()
        .flatten()
        {
            push(
                next,
                StepKind::Page,
                &super::next_sources(fetch, query),
                Request::GetVariant(query.clone()),
            );
        }
        if let Some(civic) = data(sections.civic.as_ref()) {
            for profile in &civic.profiles {
                for query in [&profile.evidence_page.next, &profile.assertion_page.next]
                    .into_iter()
                    .flatten()
                {
                    push(
                        next,
                        StepKind::Page,
                        &super::next_sources(fetch, query),
                        Request::GetVariant(query.clone()),
                    );
                }
            }
        }
        if let Some(alphagenome) = data(sections.alphagenome.as_ref()) {
            for query in alphagenome
                .scorers
                .iter()
                .filter_map(|scorer| scorer.next.as_ref())
            {
                push(
                    next,
                    StepKind::Page,
                    &super::next_sources(fetch, query),
                    Request::GetVariant(query.clone()),
                );
            }
        }
    }
}
fn data<T>(section: Option<&Section<T>>) -> Option<&T> {
    match section? {
        Section::Data { data, .. } => Some(data),
        _ => None,
    }
}
fn push(next: &mut Vec<crate::next::Step>, kind: StepKind, sources: &[Source], request: Request) {
    let action = Action::Run {
        request,
        fresh: false,
    };
    if !next.iter().any(|step| step.1 == action) {
        next.push(crate::next::Step::new(kind, sources, action));
    }
}

pub(super) fn next(
    fetch: &biomcp_fetch::Fetch,
    query: &VariantGet,
    resolution: VariantResolution,
    candidates: &[VariantHeader],
    page: Option<&biomcp_types::PageInfo>,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Vec<crate::next::Step> {
    let mut next = Vec::new();
    if resolution == VariantResolution::Candidates
        || resolution == VariantResolution::Resolved
            && candidates.iter().any(|row| {
                row.source == Source::MyVariant && row.record.id.as_str() != query.input().as_str()
                    || row
                        .sections
                        .as_ref()
                        .and_then(|s| s.population.as_ref())
                        .is_some_and(|s| {
                            matches!(
                                s,
                                Section::Unavailable {
                                    reason: biomcp_types::Reason::NoJoinId,
                                    ..
                                }
                            )
                        })
            })
    {
        for row in candidates {
            if let Some((request, sources)) =
                candidate_request(fetch, query, resolution, row, problems)
            {
                push(&mut next, StepKind::Disambiguate, &sources, request);
            }
        }
    }
    if let Some(page) = page
        && (page.offset as u64 + page.returned as u64) < page.total
    {
        next.push(crate::next::Step::run(
            StepKind::Page,
            &super::next_sources(fetch, query),
            Request::GetVariant(query.at_offset(page.offset + page.returned)),
            false,
        ));
    }
    if query.sections().is_empty() && !candidates.is_empty() {
        let section = query
            .clone()
            .with_sections(vec![biomcp_types::VariantSection::Clinvar].into());
        next.push(crate::next::Step::run(
            StepKind::Section,
            &super::next_sources(fetch, &section),
            Request::GetVariant(section),
            false,
        ));
    }
    collect(fetch, candidates, &mut next);
    if query.input().namespace() == biomcp_types::VariantNamespace::Rs {
        for gene in candidates.iter().flat_map(|row| &row.identity.genes) {
            push(
                &mut next,
                StepKind::Pivot,
                &[Source::PubMed],
                Request::SearchArticle(biomcp_types::ArticleSearch::for_gene(
                    biomcp_types::GeneInput::Text(gene.label().clone()),
                )),
            );
        }
    }
    next
}

fn candidate_request(
    fetch: &biomcp_fetch::Fetch,
    query: &VariantGet,
    resolution: VariantResolution,
    row: &VariantHeader,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Option<(Request, Vec<Source>)> {
    if super::population_identity::requested(query) {
        let (request, source) = super::population_identity::choice(row, problems)?;
        let (request, sources) = match request {
            Request::GetVariant(q) if resolution == VariantResolution::Candidates => {
                let q = q.with_sections(query.sections().into());
                let sources = super::next_sources(fetch, &q);
                (Request::GetVariant(q), sources)
            }
            request => (request, vec![source]),
        };
        return Some((request, sources));
    }
    let input = crate::next::checked(row.record.id.as_str().parse(), row.source, problems)?;
    let query = crate::next::checked(
        VariantGet::new(
            input,
            query.resolver(),
            row.identity
                .representations
                .iter()
                .find_map(|r| r.build)
                .or(query.build()),
        ),
        row.source,
        problems,
    )?;
    let sources = super::next_sources(fetch, &query);
    Some((Request::GetVariant(query), sources))
}
