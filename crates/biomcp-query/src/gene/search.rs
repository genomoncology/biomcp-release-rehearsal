use super::next;
use crate::{Ctx, Outcome, status};
use biodata::{Gene, LocusGroup, MatchKind, Text};
use biomcp_types::ReplyBody;

use biomcp_adapters::hgnc;
use biomcp_fetch::{Find, KeyKind, Leg, RequestError};
use biomcp_types::{GeneHeader, Match};
use biomcp_types::{GeneSearch, PageInfo, Source, SourceState};

pub(crate) fn search(cx: &Ctx<'_>, query: &GeneSearch) -> Outcome {
    let leg = cx
        .fetch
        .leg(&hgnc::SPEC, cx.fetch.deadline(cx.budget))
        .fresh(cx.fresh);
    let mut dropped = false;
    let mut problems = Vec::new();
    let result = collect(&leg, query, &mut problems, &mut dropped);
    let error = result.as_ref().err().copied();
    let source = status::source(Source::Hgnc, leg.finish(), error);
    let state = match (&result, source.state) {
        (Err(_), _) => crate::status::item_status(false, true, false),
        (_, SourceState::Stale) => crate::status::item_status(true, false, true),
        _ => crate::status::item_status(true, false, false),
    };
    let mut headers = result.unwrap_or_default();
    headers.retain(|(_, header)| {
        query
            .locus_group()
            .is_none_or(|group| header.locus_group == Some(group))
    });
    // Stable sort preserves published row order inside each rank and locus group.
    headers.sort_by_key(|(rank, header)| {
        (*rank, header.locus_group != Some(LocusGroup::ProteinCoding))
    });
    let mut seen = std::collections::BTreeSet::new();
    headers.retain(|(_, header)| seen.insert(header.hgnc_id));
    let total = headers.len() as u64;
    let items: Vec<_> = headers
        .into_iter()
        .skip(query.offset() as usize)
        .take(query.limit() as usize)
        .map(|(_, header)| header)
        .collect();
    let page = PageInfo {
        pool: None,
        truncated: None,
        offset: query.offset(),
        returned: items.len() as u32,
        total,
    };
    let next = next::search(query, &items, &page, &source, &mut problems);
    crate::status::finish(
        [state],
        crate::status::Parts {
            dropped,
            body: (biomcp_types::GenePage {
                items: items.into(),
            })
            .with_query(query.clone()),
            page: Some(page),
            sources: vec![source],
            problems,
            next,
            local: error.and_then(status::local),
        },
    )
}
fn collect(
    leg: &Leg<'_>,
    query: &GeneSearch,
    problems: &mut Vec<biomcp_types::Problem>,
    dropped: &mut bool,
) -> Result<Vec<(u8, GeneHeader)>, RequestError> {
    let Some(text) = query.query() else {
        let rows = hgnc::in_band(leg, query.band().map_or("", |band| band.as_str()))?;
        *dropped |= rows.refusals == biomcp_adapters::Refusal::Refused;
        status::append(problems, rows.problems);
        return Ok(rows.value.into_iter().map(|header| (0, header)).collect());
    };
    let mut result = Vec::new();
    for (rank, find, kind) in [
        (
            0,
            Find::Exact(KeyKind::Symbol, text.as_str()),
            Some(MatchKind::Symbol),
        ),
        (
            1,
            Find::Exact(KeyKind::PreviousSymbol, text.as_str()),
            Some(MatchKind::PreviousSymbol),
        ),
        (
            2,
            Find::Exact(KeyKind::Alias, text.as_str()),
            Some(MatchKind::Alias),
        ),
        (3, Find::Words(text.as_str()), None),
    ] {
        let rows = hgnc::lookup(leg, find)?;
        *dropped |= rows.refusals == biomcp_adapters::Refusal::Refused;
        status::append(problems, rows.problems);
        for gene in rows.value {
            if query.band().is_some_and(|band| {
                gene.location
                    .as_ref()
                    .is_none_or(|value| !value.as_str().starts_with(band.as_str()))
            }) {
                continue;
            }
            result.push((rank, header(gene, kind, text)));
        }
    }
    Ok(result)
}
fn header(gene: Gene, kind: Option<MatchKind>, text: &Text) -> GeneHeader {
    let matched = kind.and_then(|kind| {
        let value = match kind {
            MatchKind::Symbol => Some(gene.symbol.to_string()),
            MatchKind::PreviousSymbol => gene
                .previous_symbols
                .iter()
                .find(|v| v.as_str().eq_ignore_ascii_case(text.as_str()))
                .map(ToString::to_string),
            MatchKind::Alias => gene
                .aliases
                .iter()
                .find(|v| v.as_str().eq_ignore_ascii_case(text.as_str()))
                .map(ToString::to_string),
            _ => None,
        }?;
        Some(Match {
            kind,
            text: Text::new(&value).ok()?,
        })
    });
    GeneHeader {
        hgnc_id: gene.hgnc_id,
        symbol: gene.symbol,
        locus_group: gene.locus_group,
        location: gene.location,
        status: gene.status,
        name: gene.name,
        r#match: matched,
    }
}
