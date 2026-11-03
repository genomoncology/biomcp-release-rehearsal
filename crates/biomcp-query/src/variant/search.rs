//! Declared variant searches keep native candidates, totals and page selectors.
use crate::{Ctx, Outcome, status};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;

use biomcp_adapters::{clinvar, myvariant};
use biomcp_types::{
    PageInfo, ReplyData, Request, Source, StepKind, VariantGet, VariantPage, VariantResolver,
    VariantSearch,
};
/// Discover one source-native page; a zero result remains distinct from a failed source.
pub(crate) fn search(cx: &Ctx<'_>, query: &VariantSearch) -> Outcome {
    if query.raw_query()
        && !cx.fetch.disabled(Source::MyVariant)
        && myvariant::components::restricted(cx.fetch)
    {
        return policy_error();
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let spec = if query.uses_clinvar_selector() {
        &clinvar::SPEC
    } else {
        &myvariant::SPEC
    };
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    let terms = component_terms("clinvar", query.uses_clinvar_filters());
    let (selector, source_query) = selector(cx, query);
    let selector_terms = component_terms(selector, query.uses_dbnsfp_selector());
    let reply = match (&terms, &selector_terms) {
        (Ok(_), Ok(_)) => fetch_page(cx, &leg, query, selector, &source_query),
        (Err(error), _) | (_, Err(error)) => Err(*error),
    };
    let terms = terms.ok().flatten();
    let selector_terms = selector_terms.ok().flatten();
    let error = reply.as_ref().err().copied();
    let source = status::source(spec.source, leg.finish(), error);
    let mut out = crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: (VariantPage {
                selector_terms: selector_terms.clone(),
                filter_terms: terms.clone(),
                source_query: source_query.clone(),
                build: query.build(),
                items: Box::default(),
            })
            .with_query(query.clone()),
            page: None,
            sources: vec![source],
            problems: Vec::new(),
            next: Vec::new(),
            local: error.and_then(status::local),
        },
    );
    if let Ok(reply) = reply {
        out.include_refusals(reply.rows.refusals);
        let returned = reply.rows.value.len() as u32;
        out.next = next(
            query,
            &reply.rows.value,
            reply.consumed,
            reply.total,
            &mut out.problems,
        );
        out.page = Some(PageInfo {
            offset: query.offset(),
            returned,
            total: reply.total,
            pool: None,
            truncated: (reply.total > u64::from(VariantSearch::MAX_RESULTS)).then_some(true),
        });
        out.problems.extend(reply.rows.problems);
        out.body = (VariantPage {
            selector_terms,
            filter_terms: terms,
            source_query,
            build: query.build(),
            items: reply.rows.value.into(),
        })
        .with_query(query.clone());
    }
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
fn selector(cx: &Ctx<'_>, query: &VariantSearch) -> (&'static str, String) {
    if myvariant::components::disabled(cx.fetch, "dbnsfp") {
        ("snpeff", query.snpeff_source_query())
    } else {
        ("dbnsfp", query.source_query())
    }
}
fn fetch_page(
    cx: &Ctx<'_>,
    leg: &biomcp_fetch::Leg<'_>,
    query: &VariantSearch,
    selector: &str,
    source_query: &str,
) -> Result<SearchPage, biomcp_fetch::RequestError> {
    if query.uses_clinvar_selector() {
        return clinvar::search(leg, query).map(|page| SearchPage {
            total: page.total,
            consumed: page.consumed,
            rows: page.rows,
        });
    }
    if (query.uses_clinvar_filters() && myvariant::components::disabled(cx.fetch, "clinvar"))
        || (query.uses_dbnsfp_selector() && myvariant::components::disabled(cx.fetch, selector))
    {
        return Err(biomcp_fetch::SourceError::disabled(Source::MyVariant).into());
    }
    myvariant::search(leg, query, source_query).map(|page| SearchPage {
        total: page.total,
        consumed: page.consumed,
        rows: page.rows,
    })
}
struct SearchPage {
    total: u64,
    consumed: u32,
    rows: biomcp_adapters::Mapped<Vec<biomcp_types::VariantHeader>>,
}
fn component_terms(
    name: &str,
    selected: bool,
) -> Result<Option<biomcp_types::ComponentTerms>, biomcp_fetch::RequestError> {
    if !selected {
        return Ok(None);
    }
    let terms =
        myvariant::components::terms(name).ok_or(biomcp_fetch::InputError::Serialization)?;
    Ok(Some(myvariant::components::label(terms)?))
}
fn policy_error() -> Outcome {
    crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: ReplyData::Usage(biomcp_types::UsageError::VariantQueryPolicyError),
            page: None,
            sources: Vec::new(),
            problems: Vec::new(),
            next: Vec::new(),
            local: None,
        },
    )
}
fn next(
    query: &VariantSearch,
    rows: &[biomcp_types::VariantHeader],
    consumed: u32,
    total: u64,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Vec<crate::next::Step> {
    let mut next = Vec::new();
    for row in rows {
        if let Some(input) =
            crate::next::checked(row.record.id.as_str().parse(), row.source, problems)
            && let Some(q) = crate::next::checked(
                VariantGet::new(
                    input,
                    VariantResolver::Car,
                    row.identity.representations.iter().find_map(|r| r.build),
                ),
                row.source,
                problems,
            )
        {
            next.push(crate::next::Step::run(
                StepKind::Disambiguate,
                &[super::source(&q).source],
                Request::GetVariant(q),
                false,
            ));
        }
    }
    let offset = query.offset() + consumed;
    if u64::from(offset) < total && offset < VariantSearch::MAX_RESULTS && consumed > 0 {
        next.push(crate::next::Step::run(
            StepKind::Page,
            &[if query.uses_clinvar_selector() {
                Source::ClinVar
            } else {
                Source::MyVariant
            }],
            Request::SearchVariant(query.at_offset(offset)),
            false,
        ));
    }
    next
}
