use crate::{Ctx, status};

use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    PageInfo, Problem, ProblemCode, Reason, Section, Source, SourceOutcome, VariantGet,
    VariantHeader, VariantHotspots,
};

pub(super) fn section(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: Deadline<'_>,
) -> (
    Section<VariantHotspots>,
    Vec<SourceOutcome>,
    Vec<Problem>,
    Option<RequestError>,
    bool,
) {
    let leg = cx
        .fetch
        .leg(&biomcp_adapters::hotspots::SPEC, deadline)
        .fresh(cx.fresh);
    let genes = candidates
        .iter()
        .flat_map(|c| {
            c.identity
                .genes
                .iter()
                .chain(c.myvariant.iter().flat_map(|v| v.genes.iter()))
        })
        .filter_map(|gene| gene.label().as_str().parse::<biodata::GeneSymbol>().ok())
        .fold(Vec::new(), |mut genes, gene| {
            if !genes.contains(&gene) {
                genes.push(gene);
            }
            genes
        });
    let mut rows = Vec::new();
    let mut dropped = false;
    let mut problems = Vec::new();
    let disabled = cx.fetch.disabled(Source::CancerHotspots);
    let mut error = disabled.then_some(failure(Reason::Disabled));
    for gene in &genes {
        if disabled {
            break;
        }
        match biomcp_adapters::hotspots::gene(&leg, gene) {
            Ok(mapped) => {
                dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                rows.extend(mapped.value);
                status::append(&mut problems, mapped.problems);
            }
            Err(e) => {
                error = Some(e);
                break;
            }
        }
    }
    if genes.is_empty() && !disabled {
        error = Some(failure(Reason::NoJoinId));
    }
    if rows.is_empty() && dropped && error.is_none() {
        error = Some(failure(Reason::BadReply));
    }
    let source = status::source(Source::CancerHotspots, leg.finish(), error);
    let value = disclose(query, rows, error, &mut problems);
    (
        value,
        vec![source],
        problems,
        error.and_then(status::local),
        dropped,
    )
}

fn disclose(
    query: &VariantGet,
    rows: Vec<biomcp_types::HotspotRecord>,
    error: Option<RequestError>,
    problems: &mut Vec<Problem>,
) -> Section<VariantHotspots> {
    if let Some(error) = error {
        Section::unavailable(
            Source::CancerHotspots,
            crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
        )
    } else if rows.is_empty() {
        Section::Empty {
            source: Source::CancerHotspots,
            record: None,
        }
    } else {
        problems.push(Problem {
            source: Source::CancerHotspots,
            record: None,
            field: "transcript_consequence_join".into(),
            code: ProblemCode::ValueUnknown,
        });
        let total = rows.len() as u64;
        let records = rows
            .into_iter()
            .skip(query.section_offset() as usize)
            .take(query.section_limit() as usize)
            .collect::<Vec<_>>();
        let returned = records.len() as u32;
        let next = if (query.section_offset() as u64 + returned as u64) < total {
            Some(query.at_section_offset(query.section_offset() + returned))
        } else {
            None
        };
        Section::Data {
            source: Source::CancerHotspots,
            data: VariantHotspots {
                reason: Some(Reason::NoJoinId),
                records: records.into(),
                page: PageInfo {
                    offset: query.section_offset(),
                    returned,
                    total,
                    pool: None,
                    truncated: None,
                },
                next,
            },
        }
    }
}

fn failure(reason: Reason) -> RequestError {
    biomcp_fetch::SourceError {
        source: Source::CancerHotspots,
        reason,
        status: None,
        tls_cause: None,
        retry_after: None,
    }
    .into()
}
