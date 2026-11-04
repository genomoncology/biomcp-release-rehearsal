//! Pure typed follow-up requests; source names never become chosen drug identities.
use crate::Outcome;
use crate::next::run;
use biomcp_types::Paged;
use biomcp_types::{
    PgxAnnotations, PgxGet, PgxPageInfo, PgxRecommendations, PgxSection, PgxSource, ReplyData,
    Request, SourceState, Status, StepKind,
};
fn gene_query(
    gene: &biodata::GeneSymbol,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Option<PgxGet> {
    crate::next::checked(
        PgxGet::new(gene.clone()),
        biomcp_types::Source::Cpic,
        problems,
    )
}
fn section(
    q: &PgxGet,
    s: PgxSection,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Option<Request> {
    crate::next::checked(
        q.with_section(s, PgxSource::Cpic),
        biomcp_types::Source::Cpic,
        problems,
    )
    .map(Request::GetPgx)
}
pub(super) fn steps(request: &Request, out: &mut Outcome) -> Vec<crate::next::Step> {
    steps_with(request, out, None)
}
pub(super) fn steps_with(
    request: &Request,
    out: &mut Outcome,
    native_consumed: Option<u32>,
) -> Vec<crate::next::Step> {
    let mut next = vec![];
    let association = matches!(
        &out.body,
        ReplyData::PgxGuidelines { .. } | ReplyData::PgxAnnotations { .. }
    );
    if out.status() == Status::Failed {
        if !diplotype(request) && out.sources.iter().any(|s| s.state != SourceState::Skipped) {
            next.push(crate::next::Step::run(
                StepKind::Fresh,
                &out.sources
                    .iter()
                    .map(|source| source.source)
                    .collect::<Vec<_>>(),
                request.clone(),
                true,
            ));
        }
        return next;
    }
    success_steps(request, out, native_consumed, &mut next);
    if out.status() == Status::Partial && !diplotype(request) {
        next.push(crate::next::Step::run(
            StepKind::Fresh,
            &out.sources
                .iter()
                .map(|source| source.source)
                .collect::<Vec<_>>(),
            request.clone(),
            true,
        ));
    }
    if !association {
        next.truncate(5);
    }
    next
}
fn success_steps(
    request: &Request,
    out: &mut Outcome,
    native_consumed: Option<u32>,
    next: &mut Vec<crate::next::Step>,
) {
    let body = out.body.clone();
    match (request, &body) {
        (
            Request::SearchPgx(q),
            ReplyData::PgxPage {
                body: Some(body), ..
            },
        ) => {
            let consumed = native_consumed.unwrap_or(body.page.returned);
            let offset = body.page.offset.saturating_add(consumed);
            if consumed > 0 && u64::from(offset) < body.page.total {
                next.push(crate::next::Step::new(
                    StepKind::Page,
                    &[biomcp_types::Source::Cpic],
                    run(Request::SearchPgx(q.at_offset(offset))),
                ));
            }
            if let Some(card) = gene_query(q.gene(), &mut out.problems) {
                next.push(crate::next::Step::new(
                    StepKind::Section,
                    &[biomcp_types::Source::Cpic],
                    run(Request::GetPgx(card.clone())),
                ));
                if let Some(r) = section(&card, PgxSection::Recommendations, &mut out.problems) {
                    next.push(crate::next::Step::new(
                        StepKind::Section,
                        &[biomcp_types::Source::Cpic],
                        run(r),
                    ));
                }
            }
        }
        (Request::GetPgx(q), ReplyData::PgxCard { body: Some(_), .. }) => {
            card_steps(q, next, &mut out.problems)
        }
        (
            Request::GetPgx(q),
            ReplyData::PgxAnnotations {
                body: Some(body), ..
            },
        ) => annotation_steps(q, body, next, &mut out.problems),
        (
            Request::GetPgx(q),
            ReplyData::PgxRecommendations {
                body:
                    Some(
                        PgxRecommendations::View { page, .. }
                        | PgxRecommendations::Table { page, .. },
                    ),
                ..
            },
        ) => page_step(q, page, next, native_consumed, &mut out.problems),
        (
            Request::GetPgx(q),
            ReplyData::PgxGuidelines {
                body: Some(body), ..
            },
        ) => {
            guideline_steps(q, body, next, native_consumed, &mut out.problems);
        }
        (Request::GetPgx(q), ReplyData::PgxDiplotype { .. }) => {
            if let Some(card) = gene_query(q.gene(), &mut out.problems)
                && let Some(r) = section(&card, PgxSection::Recommendations, &mut out.problems)
            {
                next.push(crate::next::Step::new(
                    StepKind::Section,
                    &[biomcp_types::Source::Cpic],
                    run(r),
                ));
            }
        }
        _ => {}
    }
}
fn guideline_steps(
    q: &PgxGet,
    body: &biomcp_types::PgxGuidelines,
    next: &mut Vec<crate::next::Step>,
    native_consumed: Option<u32>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for gene in body.items.iter().flat_map(|r| r.genes.iter()) {
        if gene.symbol == *q.gene() || !seen.insert(gene.symbol.to_string()) {
            continue;
        }
        if let Some(query) = crate::next::checked(
            gene.symbol
                .as_str()
                .parse()
                .map_err(|_| biomcp_types::QueryInvalid::new(&["input"]))
                .and_then(|input| biomcp_types::GeneGet::new(vec![input].into())),
            biomcp_types::Source::Hgnc,
            problems,
        ) {
            next.push(crate::next::Step::new(
                StepKind::Pivot,
                &[biomcp_types::Source::Hgnc],
                run(Request::GetGene(query)),
            ));
        }
    }
    page_step(q, &body.page, next, native_consumed, problems);
}

fn page_step(
    q: &PgxGet,
    p: &PgxPageInfo,
    next: &mut Vec<crate::next::Step>,
    consumed: Option<u32>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    let consumed = consumed.unwrap_or(p.returned);
    let offset = p.offset.saturating_add(consumed);
    if consumed > 0
        && u64::from(offset) < p.total
        && let Some(query) =
            crate::next::checked(q.at_section_offset(offset), q.source().source(), problems)
    {
        next.push(crate::next::Step::new(
            StepKind::Page,
            &[q.source().source()],
            run(Request::GetPgx(query)),
        ));
    }
}

fn diplotype(request: &Request) -> bool {
    matches!(request,Request::GetPgx(q) if q.section()==Some(PgxSection::Diplotype))
}

fn annotation_steps(
    q: &PgxGet,
    body: &PgxAnnotations,
    next: &mut Vec<crate::next::Step>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    match body {
        PgxAnnotations::Clinical { items, page, .. } => {
            for row in items {
                if let Some(query) = crate::next::checked(
                    biomcp_types::PgxAnnotationGet::new(row.accession_id.clone()),
                    biomcp_types::Source::ClinPgx,
                    problems,
                ) {
                    next.push(crate::next::Step::new(
                        StepKind::Disambiguate,
                        &[biomcp_types::Source::ClinPgx],
                        run(Request::GetPgxAnnotation(query)),
                    ));
                }
            }
            page_step(q, page, next, None, problems);
        }
        PgxAnnotations::Guideline { page, .. } | PgxAnnotations::Label { page, .. } => {
            page_step(q, page, next, None, problems)
        }
    }
}

fn card_steps(
    q: &PgxGet,
    next: &mut Vec<crate::next::Step>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    for s in [PgxSection::Recommendations, PgxSection::Guidelines] {
        if let Some(r) = section(q, s, problems) {
            next.push(crate::next::Step::new(
                StepKind::Section,
                &[biomcp_types::Source::Cpic],
                run(r),
            ));
        }
    }
    if let Some(query) = crate::next::checked(
        q.with_section(PgxSection::Annotations, PgxSource::Clinpgx),
        biomcp_types::Source::ClinPgx,
        problems,
    ) {
        next.push(crate::next::Step::new(
            StepKind::Section,
            &[biomcp_types::Source::ClinPgx],
            run(Request::GetPgx(query)),
        ));
    }
}
