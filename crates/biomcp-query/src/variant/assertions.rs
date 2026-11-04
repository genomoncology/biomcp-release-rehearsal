//! Source assertions with independent row paging and exact record selection.
use crate::{Ctx, Outcome, status};
use biodata::VariantId;
use biomcp_fetch::Deadline;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    AssertionSource, PageInfo, Reason, Request, Section, StepKind, VariantAssertion,
    VariantAssertions, VariantAssertionsGet, VariantGet, VariantHeader,
};
pub(crate) fn get(cx: &Ctx<'_>, query: &VariantAssertionsGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let (section, sources, problems, local, dropped) = resolved(cx, query, deadline);
    let (status, body) = match section {
        Section::Data { data, .. } => (crate::status::item_status(true, false, dropped), data),
        Section::Empty { .. } => (
            crate::status::item_status(dropped, false, dropped),
            empty(query),
        ),
        _ => (crate::status::item_status(false, true, false), empty(query)),
    };
    let next = body
        .next
        .clone()
        .map(|q| {
            crate::next::Step::run(
                StepKind::Page,
                &[q.source().source()],
                Request::VariantAssertions(q),
                false,
            )
        })
        .into_iter()
        .collect();
    command.finish(deadline.remaining());
    crate::status::finish(
        [status],
        crate::status::Parts {
            dropped,
            body: body.with_query(query.clone()),
            page: None,
            sources,
            problems,
            next,
            local,
        },
    )
}
/// Section hooks receive one candidate; source input IDs can be checked caller selectors.
pub(super) fn section(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: Deadline<'_>,
    source: AssertionSource,
) -> super::sections::Result<VariantAssertions> {
    let id = candidates.first().and_then(|c| {
        c.requested
            .as_ref()
            .into_iter()
            .chain(c.identity.ids.iter())
            .find(|id| {
                matches!(
                    (source, id),
                    (
                        AssertionSource::Clinvar,
                        VariantId::ClinVarVariation(_) | VariantId::Vcv(_)
                    ) | (AssertionSource::Erepo, VariantId::Ca(_))
                )
            })
    });
    let Some(id) = id else {
        return (
            Section::unavailable(source.source(), Reason::NoJoinId),
            Vec::new(),
            Vec::new(),
            None,
            false,
        );
    };
    let q = match VariantAssertionsGet::from_identity(
        id.clone(),
        query.assertion_limit(source),
        query.section_offset(),
    ) {
        Ok(query) => query,
        Err(_) => {
            return (
                Section::unavailable(source.source(), Reason::NoJoinId),
                Vec::new(),
                Vec::new(),
                None,
                false,
            );
        }
    };
    read(cx, &q, deadline)
}
fn resolved(
    cx: &Ctx<'_>,
    query: &VariantAssertionsGet,
    deadline: Deadline<'_>,
) -> super::sections::Result<VariantAssertions> {
    let native = query.input().identity().is_some_and(|id| {
        matches!(
            (query.source(), id),
            (
                AssertionSource::Clinvar,
                VariantId::ClinVarVariation(_) | VariantId::Vcv(_)
            ) | (AssertionSource::Erepo, VariantId::Ca(_))
        )
    });
    if native {
        return read(cx, query, deadline);
    }
    let identity = match VariantGet::new(
        query.input().clone(),
        biomcp_types::VariantResolver::Car,
        query.build(),
    ) {
        Ok(identity) => identity,
        Err(_) => return missing(query, Vec::new(), Vec::new(), None, false),
    };
    let found = super::identity::read(cx, &identity, deadline);
    let input = match selection(query, &found.rows) {
        Ok(input) => input,
        Err(_) => {
            return missing(
                query,
                found.sources,
                found.problems,
                found.local,
                found.dropped,
            );
        }
    };
    let converted = query.with_input(input);
    let (section, sources, problems, local, dropped) =
        read(cx, &converted, deadline.after_stage(found.elapsed));
    let mut all_sources = found.sources;
    all_sources.extend(sources);
    let mut all_problems = found.problems;
    all_problems.extend(problems);
    (
        section,
        all_sources,
        all_problems,
        found.local.or(local),
        found.dropped || dropped,
    )
}
fn selection(
    query: &VariantAssertionsGet,
    candidates: &[VariantHeader],
) -> Result<biomcp_types::VariantInput, biomcp_types::QueryInvalid> {
    let [row] = candidates else {
        return Err(biomcp_types::QueryInvalid::new(&["input"]));
    };
    let mut ids = row
        .identity
        .ids
        .iter()
        .filter(|id| {
            matches!(
                (query.source(), id),
                (
                    AssertionSource::Clinvar,
                    VariantId::ClinVarVariation(_) | VariantId::Vcv(_)
                ) | (AssertionSource::Erepo, VariantId::Ca(_))
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    if ids.len() != 1 {
        return Err(biomcp_types::QueryInvalid::new(&["input"]));
    }
    let id = ids
        .pop_first()
        .ok_or_else(|| biomcp_types::QueryInvalid::new(&["input"]))?;
    biomcp_types::VariantInput::from_identity(id.clone())
        .map_err(|_| biomcp_types::QueryInvalid::new(&["input"]))
}
fn missing(
    query: &VariantAssertionsGet,
    sources: Vec<biomcp_types::SourceOutcome>,
    mut problems: Vec<biomcp_types::Problem>,
    local: Option<biomcp_fetch::RequestError>,
    dropped: bool,
) -> super::sections::Result<VariantAssertions> {
    problems.push(super::identity::missing(query.source().source()));
    (
        Section::unavailable(query.source().source(), Reason::NoJoinId),
        sources,
        problems,
        local,
        dropped,
    )
}
fn read(
    cx: &Ctx<'_>,
    query: &VariantAssertionsGet,
    deadline: Deadline<'_>,
) -> super::sections::Result<VariantAssertions> {
    let spec = match query.source() {
        AssertionSource::Clinvar => &biomcp_adapters::clinvar::SPEC,
        AssertionSource::Erepo => &biomcp_adapters::erepo::SPEC,
    };
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    let native = query.input().source_id();
    let input = native.as_ref();
    let result = match query.source() {
        AssertionSource::Clinvar => biomcp_adapters::clinvar::assertions(
            &leg,
            input.strip_prefix("clinvar:").unwrap_or(input),
            query
                .assertion()
                .map(|id| (id, query.version().map(|v| v.as_str()))),
        ),
        AssertionSource::Erepo => biomcp_adapters::erepo::assertions(
            &leg,
            input,
            query
                .assertion()
                .map(|id| (id, query.version().map(|v| v.as_str()))),
        ),
    };
    let error = result.as_ref().err().copied();
    let outcome = status::source(spec.source, leg.finish(), error);
    let mut dropped = false;
    let mut problems = Vec::new();
    let section = match result {
        Ok(rows) => {
            dropped = rows.refusals == biomcp_adapters::Refusal::Refused;
            problems = rows.problems;
            let data = page(query, rows.value);
            if data.page.total == 0 {
                Section::Empty {
                    source: spec.source,
                    record: None,
                }
            } else {
                Section::Data {
                    source: spec.source,
                    data,
                }
            }
        }
        Err(error) => Section::unavailable(
            spec.source,
            crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
        ),
    };
    (
        section,
        vec![outcome],
        problems,
        error.and_then(status::local),
        dropped,
    )
}
fn page(query: &VariantAssertionsGet, mut rows: Vec<VariantAssertion>) -> VariantAssertions {
    if let Some(id) = query.assertion() {
        let id = id.to_string();
        rows.retain(|r| {
            r.attribution().is_some_and(|a| {
                a.record.as_ref().map(|record| record.as_str()) == Some(id.as_str())
                    && query
                        .version()
                        .is_none_or(|v| a.version.as_ref() == Some(v))
            })
        });
    }
    let total = rows.len() as u64;
    let mut rows = rows
        .into_iter()
        .skip(query.offset() as usize)
        .take(query.limit() as usize)
        .collect::<Vec<_>>();
    if query.assertion().is_none() {
        for row in &mut rows {
            if !row.statement_unmapped {
                row.detail = None;
            }
            if let Some(s) = &mut row.statement {
                s.citations = Box::default();
                s.criteria = Box::default();
            }
        }
    }
    let returned = rows.len() as u32;
    VariantAssertions {
        assertions: rows.into(),
        page: PageInfo {
            offset: query.offset(),
            returned,
            total,
            pool: None,
            truncated: None,
        },
        next: ((query.offset() as u64 + returned as u64) < total)
            .then(|| query.at_offset(query.offset() + returned)),
    }
}
fn empty(query: &VariantAssertionsGet) -> VariantAssertions {
    VariantAssertions {
        assertions: Box::default(),
        page: PageInfo {
            offset: query.offset(),
            returned: 0,
            total: 0,
            pool: None,
            truncated: None,
        },
        next: None,
    }
}
