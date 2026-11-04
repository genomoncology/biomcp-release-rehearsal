use biodata::{Gene, LocusGroup, MatchKind, Text};

use biomcp_adapters::{Mapped, hgnc};
use biomcp_fetch::{Find, KeyKind, Leg, RequestError};
use biomcp_types::Problem;
use biomcp_types::{GeneHeader, GeneInput, GeneResolution, Match};
use std::collections::BTreeMap;

/// Resolve an input against approved HGNC identity before withdrawn identity.
/// Rejected source fields remain visible even when no accepted record remains.
pub(crate) fn resolve(
    leg: &Leg<'_>,
    input: &GeneInput,
) -> Result<Mapped<GeneResolution>, RequestError> {
    let mut problems = Vec::new();
    let mut refusals = biomcp_adapters::Refusal::None;
    let mut candidates = Vec::new();
    match input {
        GeneInput::HgncId(id) => collect(
            leg,
            KeyKind::Id,
            &id.to_string(),
            MatchKind::HgncId,
            &mut candidates,
            &mut problems,
            &mut refusals,
        )?,
        GeneInput::EntrezId(id) => collect(
            leg,
            KeyKind::Xref,
            &format!("entrez:{id}"),
            MatchKind::EntrezId,
            &mut candidates,
            &mut problems,
            &mut refusals,
        )?,
        GeneInput::EnsemblId(id) => collect(
            leg,
            KeyKind::Xref,
            &format!("ensembl:{}", id.base()),
            MatchKind::EnsemblId,
            &mut candidates,
            &mut problems,
            &mut refusals,
        )?,
        GeneInput::Text(text) => {
            collect(
                leg,
                KeyKind::Symbol,
                text.as_str(),
                MatchKind::Symbol,
                &mut candidates,
                &mut problems,
                &mut refusals,
            )?;
            if candidates.is_empty() {
                collect(
                    leg,
                    KeyKind::PreviousSymbol,
                    text.as_str(),
                    MatchKind::PreviousSymbol,
                    &mut candidates,
                    &mut problems,
                    &mut refusals,
                )?;
                collect(
                    leg,
                    KeyKind::Alias,
                    text.as_str(),
                    MatchKind::Alias,
                    &mut candidates,
                    &mut problems,
                    &mut refusals,
                )?;
            }
        }
    }
    let value = if candidates.is_empty() {
        withdrawn(leg, input, &mut problems, &mut refusals)?
    } else {
        resolved(candidates)
    };
    Ok(Mapped {
        refusals,
        value,
        problems,
    })
}
fn withdrawn(
    leg: &Leg<'_>,
    input: &GeneInput,
    problems: &mut Vec<Problem>,
    refusals: &mut biomcp_adapters::Refusal,
) -> Result<GeneResolution, RequestError> {
    let withdrawn = hgnc::withdrawn(leg, input)?;
    refusals.include(withdrawn.refusals);
    append_problems(problems, withdrawn.problems);
    Ok(match withdrawn.value.into_iter().next() {
        Some(row) => GeneResolution::Withdrawn {
            hgnc_id: row.hgnc_id,
            symbol: row.symbol,
            hgnc_status: row.hgnc_status,
            merged_into: row.merged_into,
        },
        None => GeneResolution::NotFound {},
    })
}

fn collect(
    leg: &Leg<'_>,
    key: KeyKind,
    text: &str,
    kind: MatchKind,
    candidates: &mut Vec<(Gene, Match)>,
    problems: &mut Vec<Problem>,
    refusals: &mut biomcp_adapters::Refusal,
) -> Result<(), RequestError> {
    let rows = hgnc::lookup(leg, Find::Exact(key, text))?;
    refusals.include(rows.refusals);
    append_problems(problems, rows.problems);
    candidates.extend(rows.value.into_iter().filter_map(|gene| {
        let text = match_text(&gene, kind, text)?;
        Some((gene, Match { kind, text }))
    }));
    Ok(())
}
fn append_problems(target: &mut Vec<Problem>, values: Vec<Problem>) {
    for problem in values {
        if !target.contains(&problem) {
            target.push(problem);
        }
    }
}
fn match_text(gene: &Gene, kind: MatchKind, input: &str) -> Option<Text> {
    let value = match kind {
        MatchKind::Symbol => gene.symbol.to_string(),
        MatchKind::PreviousSymbol => gene
            .previous_symbols
            .iter()
            .find(|v| v.as_str().eq_ignore_ascii_case(input))?
            .to_string(),
        MatchKind::Alias => gene
            .aliases
            .iter()
            .find(|v| v.as_str().eq_ignore_ascii_case(input))?
            .to_string(),
        MatchKind::HgncId => gene.hgnc_id.to_string(),
        MatchKind::EntrezId => gene.entrez_id?.to_string(),
        MatchKind::EnsemblId => gene.ensembl_id.as_ref()?.to_string(),
        _ => return None,
    };
    Text::new(&value).ok()
}
fn kind_rank(kind: MatchKind) -> u8 {
    match kind {
        MatchKind::PreviousSymbol => 1,
        MatchKind::Alias => 2,
        _ => 0,
    }
}
fn resolved(candidates: Vec<(Gene, Match)>) -> GeneResolution {
    let mut distinct = BTreeMap::new();
    for (gene, matched) in candidates {
        // Previous-symbol rows are collected before aliases, so first wins.
        distinct.entry(gene.hgnc_id).or_insert((gene, matched));
    }
    let mut values: Vec<_> = distinct.into_values().collect();
    values.sort_by_key(|(gene, matched)| {
        (
            kind_rank(matched.kind),
            gene.locus_group != Some(LocusGroup::ProteinCoding),
            gene.hgnc_id,
        )
    });
    if values.len() == 1
        && let Some((gene, matched)) = values.pop()
    {
        return GeneResolution::Resolved {
            gene: Box::new(gene),
            r#match: matched,
        };
    }
    GeneResolution::Ambiguous {
        candidates: values
            .into_iter()
            .map(|(gene, matched)| GeneHeader {
                hgnc_id: gene.hgnc_id,
                symbol: gene.symbol,
                locus_group: gene.locus_group,
                location: gene.location,
                status: gene.status,
                name: gene.name,
                r#match: Some(matched),
            })
            .collect(),
    }
}
