use super::run;
use biodata::PathwayEventRef;
use biomcp_types::Paged;
use biomcp_types::{
    GeneGet, GeneInput, PathwayEvents, PathwayEventsQuery, PathwayGenes, PathwayGenesQuery,
    PathwayGet, Request, StepKind,
};
type Steps = Vec<crate::next::Step>;
pub(super) fn genes(
    body: &PathwayGenes,
    limit: u32,
    details: bool,
    next: &mut Steps,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    let mut seen = Vec::new();
    for row in &body.items {
        if let Some(id) = row.gene
            && !seen.contains(&id)
        {
            seen.push(id);
            if let Some(query) = crate::next::checked(
                GeneGet::new(vec![GeneInput::HgncId(id)].into()),
                biomcp_types::Source::Reactome,
                problems,
            ) {
                next.push(crate::next::Step::new(
                    StepKind::Pivot,
                    &[biomcp_types::Source::Hgnc],
                    run(Request::GetGene(query)),
                ));
            }
        }
    }
    if let Some(offset) = body.page.next_offset
        && let Some(pool) = &body.page.pool
        && let Some(query) = crate::next::checked(
            PathwayGenesQuery::new(body.header.id.clone(), limit),
            biomcp_types::Source::Reactome,
            problems,
        )
    {
        let query = if details { query.with_details() } else { query };
        next.push(crate::next::Step::new(
            StepKind::Page,
            &[super::next::identity(&body.header.id)],
            run(Request::PathwayGenes(
                query.at_offset(offset).with_pool(pool.id.clone()),
            )),
        ));
    }
    if matches!(body.header.id, biodata::PathwayId::Reactome(_))
        && !details
        && !body.items.is_empty()
        && let Some(pool) = &body.page.pool
        && let Some(query) = crate::next::checked(
            PathwayGenesQuery::new(body.header.id.clone(), limit),
            biomcp_types::Source::Reactome,
            problems,
        )
    {
        next.push(crate::next::Step::new(
            StepKind::Section,
            &[biomcp_types::Source::Reactome],
            run(Request::PathwayGenes(
                query
                    .with_details()
                    .at_offset(body.page.offset)
                    .with_pool(pool.id.clone()),
            )),
        ));
    }
}

pub(super) fn events(
    body: &PathwayEvents,
    limit: u32,
    next: &mut Steps,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    if let Some(offset) = body.page.next_offset
        && let Some(pool) = &body.page.pool
        && let Some(query) = crate::next::checked(
            PathwayEventsQuery::new(body.header.id.clone(), limit),
            biomcp_types::Source::Reactome,
            problems,
        )
    {
        next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::Reactome],
            run(Request::PathwayEvents(
                query.at_offset(offset).with_pool(pool.id.clone()),
            )),
        ));
    }
    for row in &body.items {
        if let PathwayEventRef::Object {
            id: Some(id),
            schema_class: Some(class),
            ..
        } = row
            && matches!(class.as_str(), "Pathway" | "TopLevelPathway")
        {
            next.push(crate::next::Step::new(
                StepKind::Disambiguate,
                &[biomcp_types::Source::Reactome],
                run(Request::GetPathway(PathwayGet::new(
                    biodata::PathwayId::Reactome(id.clone()),
                ))),
            ));
            if next.iter().filter(|step| step.0 != StepKind::Page).count() >= 3 {
                break;
            }
        }
    }
}
