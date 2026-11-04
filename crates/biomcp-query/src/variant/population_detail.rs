//! Native ancestry pagination reuses the same identity and gnomAD requests.
use crate::{Ctx, Outcome, status};
use biodata::PopulationDataType;
use biomcp_adapters::gnomad;
use biomcp_fetch::RequestError;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    GnomadCounts, PageInfo, Reason, Request, Section, Source, StepKind, VariantPopulationQuery,
    VariantPopulationResult,
};

pub(crate) fn get(cx: &Ctx<'_>, query: &VariantPopulationQuery) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut read = read(cx, query, deadline);
    let total = read.groups.len() as u64;
    let groups: Box<[_]> = read
        .groups
        .into_iter()
        .skip(query.offset() as usize)
        .take(query.limit() as usize)
        .collect();
    let returned = groups.len() as u32;
    let page = query.detail().then_some(PageInfo {
        offset: query.offset(),
        returned,
        total,
        pool: None,
        truncated: None,
    });
    let mut next = Vec::new();
    if query.offset() as u64 + u64::from(returned) < total {
        next.push(crate::next::Step::run(
            StepKind::Page,
            &[biomcp_types::Source::Gnomad],
            Request::VariantPopulation(query.at_offset(query.offset() + returned)),
            false,
        ));
    }
    next.extend(choices(query, &read.candidates, &mut read.problems));
    let status = match &read.population {
        Section::Data { .. } => status::item_status(true, false, read.dropped),
        _ if read.candidates.len() > 1 => status::item_status(true, false, read.dropped),
        Section::Empty { .. } => status::item_status(read.dropped, false, read.dropped),
        _ => status::item_status(false, true, false),
    };
    let out = crate::status::finish(
        [status],
        crate::status::Parts {
            dropped: read.dropped,
            body: (VariantPopulationResult {
                population: read.population,
                groups,
                candidates: std::mem::take(&mut read.candidates).into(),
            })
            .with_query(query.clone()),
            page,
            sources: read.sources,
            problems: read.problems,
            next,
            local: read.local,
        },
    );
    command.finish(deadline.remaining());
    out
}
struct PopulationRead {
    dropped: bool,
    population: Section<biomcp_types::VariantPopulation>,
    groups: Vec<GnomadCounts>,
    candidates: Vec<biomcp_types::VariantHeader>,
    sources: Vec<biomcp_types::SourceOutcome>,
    problems: Vec<biomcp_types::Problem>,
    local: Option<RequestError>,
}
fn read(
    cx: &Ctx<'_>,
    query: &VariantPopulationQuery,
    deadline: biomcp_fetch::Deadline<'_>,
) -> PopulationRead {
    let mut read = PopulationRead {
        dropped: false,
        population: Section::unavailable(Source::Gnomad, Reason::NoJoinId),
        groups: Vec::new(),
        candidates: Vec::new(),
        sources: Vec::new(),
        problems: Vec::new(),
        local: None,
    };
    if cx.fetch.disabled(Source::Gnomad) {
        read.population = Section::unavailable(Source::Gnomad, Reason::Disabled);
        return read;
    }
    if let Ok(identity) = query.identity() {
        let found = super::identity::read(cx, &identity, deadline);
        read.dropped = found.dropped;
        read.sources = found.sources;
        read.problems = found.problems;
        read.local = found.local;
        read.candidates = found.rows;
        if let Some(reason) = found.reason {
            read.population = Section::unavailable(Source::Gnomad, reason);
            return read;
        }
    }
    let Some(id) = super::population::join(&read.candidates) else {
        if read.candidates.len() < 2 {
            read.problems.push(biomcp_types::Problem {
                source: Source::Gnomad,
                record: None,
                field: "GRCh38_allele".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
        }
        read.problems.push(super::identity::missing(Source::Gnomad));
        return read;
    };
    let (section, groups, source, error, dropped) = load(
        cx,
        query,
        &id,
        &read.candidates,
        &mut read.problems,
        deadline,
    );
    read.dropped |= dropped;
    read.problems.extend(projection_problems(&section, &groups));
    read.sources.push(source);
    read.population = section;
    read.groups = groups;
    read.local = read.local.or(error);
    read.candidates.clear();
    read
}
fn choices(
    query: &VariantPopulationQuery,
    candidates: &[biomcp_types::VariantHeader],
    problems: &mut Vec<biomcp_types::Problem>,
) -> Vec<crate::next::Step> {
    candidates
        .iter()
        .filter_map(|row| {
            let (request, source) = super::identity::choice(row, problems)?;
            let (request, sources) = match request {
                Request::GetVariant(identity) if candidates.len() > 1 => {
                    let query = crate::next::checked(
                        query.with_identity(identity.input().clone(), identity.build()),
                        row.source,
                        problems,
                    )?;
                    (
                        Request::VariantPopulation(query),
                        vec![source, biomcp_types::Source::Gnomad],
                    )
                }
                request => (request, vec![source]),
            };
            Some(crate::next::Step::run(
                StepKind::Disambiguate,
                &sources,
                request,
                false,
            ))
        })
        .collect()
}
fn load(
    cx: &Ctx<'_>,
    query: &VariantPopulationQuery,
    id: &str,
    candidates: &[biomcp_types::VariantHeader],
    problems: &mut Vec<biomcp_types::Problem>,
    deadline: biomcp_fetch::Deadline<'_>,
) -> (
    Section<biomcp_types::VariantPopulation>,
    Vec<GnomadCounts>,
    biomcp_types::SourceOutcome,
    Option<RequestError>,
    bool,
) {
    let leg = cx.fetch.leg(&gnomad::SPEC, deadline).fresh(cx.fresh);
    let mut groups = Vec::new();
    let mut dropped = false;
    let result = gnomad::variant::population(&leg, id).and_then(|raw| {
        dropped |= raw.refusals == biomcp_adapters::Refusal::Refused;
        problems.extend(raw.problems);
        raw.value
            .map(|raw| {
                let summary = super::population::summary(&raw, candidates, problems)?;
                if query.detail() {
                    groups = ancestry(&raw);
                }
                Ok(summary)
            })
            .transpose()
    });
    let error = result.as_ref().err().copied();
    let source = status::source(Source::Gnomad, leg.finish(), error);
    let section = match result {
        Ok(Some(data)) => Section::Data {
            source: Source::Gnomad,
            data,
        },
        Ok(None) => Section::Empty {
            source: Source::Gnomad,
            record: None,
        },
        Err(error) => Section::unavailable(
            Source::Gnomad,
            crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
        ),
    };
    (
        section,
        groups,
        source,
        error.and_then(status::local),
        dropped,
    )
}

fn ancestry(raw: &gnomad::variant::Variant) -> Vec<GnomadCounts> {
    let mut groups = Vec::new();
    for (data_type, data) in [
        (PopulationDataType::Exomes, &raw.exome),
        (PopulationDataType::Genomes, &raw.genome),
    ] {
        if let Some(data) = data {
            for group in &data.populations {
                groups.push(super::population::counts(data, data_type, Some(group)));
            }
        }
    }
    groups
}

fn projection_problems(
    section: &Section<biomcp_types::VariantPopulation>,
    groups: &[GnomadCounts],
) -> Vec<biomcp_types::Problem> {
    let mut problems = Vec::new();
    if let Section::Data { .. } = &section {
        for group in groups {
            if group.group_id.is_some() && group.ancestry_group.is_none() && group.sex.is_none() {
                problems.push(biomcp_types::Problem {
                    source: Source::Gnomad,
                    record: group.group_id.clone(),
                    field: "population_group".into(),
                    code: biomcp_types::ProblemCode::ValueUnknown,
                });
            }
        }
    }
    problems
}
