use crate::{Ctx, Outcome, status};
use biomcp_adapters::seer;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DiseaseGet, DiseaseIncidence, DiseaseSearch, DiseaseSection, DiseaseSites, DiseaseSurvival,
    Request, Source, StepKind,
};

/// Page the native site catalog without selecting an ontology identity.
pub(crate) fn search(cx: &Ctx<'_>, query: &DiseaseSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&seer::SPEC, deadline).fresh(cx.fresh);
    let reply = seer::catalog_named(&leg, query.text(), query.limit(), query.offset());
    let error = reply.as_ref().err().copied();
    let mut out = empty(
        (DiseaseSites {
            items: Box::default(),
            components: Box::default(),
            fetched: None,
            selectors: Box::default(),
        })
        .with_query(query.clone()),
    );
    out.sources
        .push(status::source(Source::Seer, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok(page) = reply {
        out.include_refusals(page.window.refusals());
        let total = page.window.total().unwrap_or(0);
        let returned = page.window.rows.len() as u32;
        out.page = Some(page_info(query.offset(), returned, total));
        if let Some(offset) = page.window.next_offset() {
            out.next.push(crate::next::Step::new(
                StepKind::Page,
                &[biomcp_types::Source::Seer],
                run(Request::SearchDisease(query.at_offset(offset))),
            ));
        }
        if query.text().is_some() && total == 0 {
            list_sites(&mut out);
        }
        out.problems.extend(page.problems);
        out.body = (DiseaseSites {
            items: page.window.rows.into(),
            components: Box::default(),
            fetched: fetched(page.metadata.fetched),
            selectors: page.selectors,
        })
        .with_query(query.clone());
        crate::status::search(&mut out);
    }
    command.finish(deadline.remaining());
    out
}
/// Preserve source stage or annual rows behind an explicit native site selection.
pub(crate) fn get(cx: &Ctx<'_>, query: &DiseaseGet, id: &biodata::Text) -> Outcome {
    let mut out = empty_get(query);
    let mut selection = match seer::selection(query, id) {
        Ok(selection) => selection,
        Err(error) => {
            return empty(biomcp_types::ReplyData::Usage(
                crate::prepare::selector_error(error),
            ));
        }
    };
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&seer::SPEC, deadline).fresh(cx.fresh);
    if !id.as_str().bytes().all(|b| b.is_ascii_digit()) {
        let (site, discovered, error) = named_site(&leg, id, query, out);
        out = discovered;
        if let Some(site) = site {
            selection.site = site;
        } else {
            out.sources
                .push(status::source(Source::Seer, leg.finish(), error));
            out.local = error.and_then(status::local);
            crate::status::search(&mut out);
            if error.is_none() && out.page.as_ref().is_some_and(|page| page.total == 0) {
                crate::status::item(&mut out, false, false, false);
            }
            command.finish(deadline.remaining());
            return out;
        }
    }
    let reply = seer::statistics(
        &leg,
        &selection,
        query.section_limit(),
        query.section_offset(),
    );
    let error = reply.as_ref().err().copied();
    out.sources
        .push(status::source(Source::Seer, leg.finish(), error));
    match reply {
        Ok(page) => include_statistics(&mut out, query, page),
        Err(error) => out.local = status::local(error),
    }
    super::page::record_problem(&mut out, query, Source::Seer);
    command.finish(deadline.remaining());
    out
}
fn fetched(value: Option<i64>) -> Option<u64> {
    value.and_then(|value| u64::try_from(value).ok())
}
use crate::next::run;
pub(super) use crate::status::empty;
pub(super) use crate::status::page_info;

fn empty_get(query: &DiseaseGet) -> Outcome {
    let survival = query.section() == Some(DiseaseSection::Survival);
    let body = if survival {
        (DiseaseSurvival {
            items: Box::default(),
            components: Box::default(),
            fetched: None,
        })
        .with_query(query.clone())
    } else {
        (DiseaseIncidence {
            items: Box::default(),
            components: Box::default(),
            fetched: None,
        })
        .with_query(query.clone())
    };
    empty(body)
}

fn named_site(
    leg: &biomcp_fetch::Leg<'_>,
    id: &biodata::Text,
    query: &DiseaseGet,
    mut out: Outcome,
) -> (
    Option<biodata::Text>,
    Outcome,
    Option<biomcp_fetch::RequestError>,
) {
    match seer::catalog_named(leg, Some(id.as_str()), u32::MAX, 0) {
        Ok(page) => {
            let exact: Vec<_> = page
                .window
                .rows
                .iter()
                .filter(|site| {
                    site.label.as_ref().is_some_and(|label| {
                        label.as_str().eq_ignore_ascii_case(id.as_str().trim())
                    })
                })
                .collect();
            let selected = match exact.as_slice() {
                [site] => Some(site.code.clone()),
                _ => match page.window.rows.as_slice() {
                    [site]
                        if site
                            .label
                            .as_ref()
                            .is_some_and(|label| whole_words(label.as_str(), id.as_str())) =>
                    {
                        Some(site.code.clone())
                    }
                    _ => None,
                },
            };
            if page.window.refusals() == biomcp_adapters::Refusal::None
                && page.window.rows.iter().all(|site| site.label.is_some())
                && let Some(site) = selected
            {
                return (Some(site), out, None);
            }
            let search = match DiseaseSearch::seer_named(id.as_str()) {
                Ok(search) => search,
                Err(_) => {
                    out.body =
                        biomcp_types::ReplyData::Usage(biomcp_types::UsageError::arguments());
                    return (None, out, None);
                }
            };
            (None, candidates(query, search, page, out), None)
        }
        Err(error) => (None, out, Some(error)),
    }
}

/// A lone match inside a longer word, such as "ear" in "Heart", stays a candidate.
fn whole_words(label: &str, text: &str) -> bool {
    let label = label.to_lowercase();
    let text = text.trim().to_lowercase();
    label.match_indices(&text).any(|(at, _)| {
        let before = label[..at].chars().next_back();
        let after = label[at + text.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// Send a name that matched nothing to the publisher's own site names.
fn list_sites(out: &mut Outcome) {
    if let Some(catalog) = crate::next::checked(
        DiseaseSearch::seer_widest(),
        Source::Seer,
        &mut out.problems,
    ) {
        out.next.push(crate::next::Step::new(
            StepKind::Disambiguate,
            &[Source::Seer],
            run(Request::SearchDisease(catalog)),
        ));
    }
}

fn include_statistics(
    out: &mut Outcome,
    query: &DiseaseGet,
    page: seer::Page<biodata::DiseaseStatistic>,
) {
    out.include_refusals(page.window.refusals());
    let total = page.window.total().unwrap_or(0);
    let returned = page.window.rows.len() as u32;
    out.page = Some(page_info(query.section_offset(), returned, total));
    if let Some(offset) = page.window.next_offset() {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::Seer],
            run(Request::GetDisease(query.at_section_offset(offset))),
        ));
    }
    if let Some(catalog) = crate::next::checked(
        biomcp_types::DiseaseSearch::seer(),
        biomcp_types::Source::Seer,
        &mut out.problems,
    ) {
        out.next.push(crate::next::Step::new(
            StepKind::Disambiguate,
            &[biomcp_types::Source::Seer],
            run(Request::SearchDisease(catalog)),
        ));
    }
    out.problems.extend(page.problems);
    out.body = if query.section() == Some(DiseaseSection::Survival) {
        (DiseaseSurvival {
            items: page.window.rows.into(),
            components: Box::default(),
            fetched: fetched(page.metadata.fetched),
        })
        .with_query(query.clone())
    } else {
        (DiseaseIncidence {
            items: page.window.rows.into(),
            components: Box::default(),
            fetched: fetched(page.metadata.fetched),
        })
        .with_query(query.clone())
    };
    crate::status::item(out, total != 0, false, false);
}

fn candidates(
    query: &DiseaseGet,
    search: DiseaseSearch,
    mut page: seer::Page<biomcp_types::DiseaseSite>,
    mut out: Outcome,
) -> Outcome {
    out.include_refusals(page.window.refusals());
    out.problems.append(&mut page.problems);
    let total = page.window.rows.len() as u32;
    out.page = Some(page_info(0, total.min(search.limit()), u64::from(total)));
    if total > search.limit() {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[Source::Seer],
            run(Request::SearchDisease(search.at_offset(search.limit()))),
        ));
    }
    if total == 0 {
        list_sites(&mut out);
    }
    page.window.rows.truncate(search.limit() as usize);
    for site in &page.window.rows {
        if let Some(selected) = crate::next::checked(
            query.with_seer_site(site.code.clone()),
            Source::Seer,
            &mut out.problems,
        ) {
            out.next.push(crate::next::Step::new(
                StepKind::Disambiguate,
                &[Source::Seer],
                run(Request::GetDisease(selected)),
            ));
        }
    }
    out.body = (DiseaseSites {
        items: page.window.rows.into(),
        components: Box::default(),
        fetched: fetched(page.metadata.fetched),
        selectors: page.selectors,
    })
    .with_query(search);
    out
}
