//! Joins use the GPML Xref's native namespace and value, never its label.
use crate::{Ctx, status};
use biodata::{Gene, PathwayMember};
use biomcp_adapters::hgnc;
use biomcp_fetch::{Deadline, Find, KeyKind};
use biomcp_types::{Problem, ProblemCode, Source, SourceOutcome};

pub(super) fn resolve(
    cx: &Ctx<'_>,
    rows: &mut [PathwayMember],
    deadline: Deadline<'_>,
    sources: &mut Vec<SourceOutcome>,
    problems: &mut Vec<Problem>,
) {
    let mut candidates = Vec::new();
    for kind in [KeyKind::Xref, KeyKind::Id, KeyKind::Symbol] {
        let keys: Vec<_> = rows
            .iter()
            .filter_map(key)
            .filter(|(k, _)| std::mem::discriminant(k) == std::mem::discriminant(&kind))
            .map(|(_, v)| v)
            .collect();
        if keys.is_empty() {
            continue;
        }
        let names: Vec<_> = keys.iter().map(String::as_str).collect();
        let leg = cx.fetch.leg(&hgnc::SPEC, deadline).fresh(cx.fresh);
        let mapped = hgnc::lookup(&leg, Find::AnyExact(kind, &names));
        let outcome = status::source(Source::Hgnc, leg.finish(), mapped.as_ref().err().copied());
        if let Some(existing) = sources.iter_mut().find(|s| s.source == Source::Hgnc) {
            status::retain_source_state(existing, &outcome);
        } else {
            sources.push(outcome);
        }
        if let Ok(mapped) = mapped {
            candidates.extend(mapped.value);
            problems.extend(mapped.problems);
        }
    }
    for row in rows {
        let mut genes = candidates
            .iter()
            .filter(|gene| exact(row, gene))
            .map(|g| g.hgnc_id)
            .collect::<Vec<_>>();
        genes.sort();
        genes.dedup();
        row.gene = (genes.len() == 1).then(|| genes.first().copied()).flatten();
        if row.gene.is_none() {
            problems.push(Problem {
                source: Source::WikiPathways,
                record: row.node_id.clone(),
                field: "genes.gene".into(),
                code: ProblemCode::ValueUnknown,
            });
        }
    }
}
fn key(row: &PathwayMember) -> Option<(KeyKind, String)> {
    let value = row.reference.id.as_str();
    match row.reference.namespace.as_deref()? {
        "Entrez Gene" if value.parse::<biodata::NcbiGeneId>().is_ok() => {
            Some((KeyKind::Xref, format!("entrez:{value}")))
        }
        "Ensembl" if value.parse::<biodata::EnsemblGeneId>().is_ok() => {
            Some((KeyKind::Xref, format!("ensembl:{value}")))
        }
        "HGNC" if hgnc_id(value).is_some() => Some((KeyKind::Id, hgnc_id(value)?.to_string())),
        "HGNC" if value.parse::<biodata::GeneSymbol>().is_ok() => {
            Some((KeyKind::Symbol, value.into()))
        }
        _ => None,
    }
}
fn exact(row: &PathwayMember, gene: &Gene) -> bool {
    let value = row.reference.id.as_str();
    match row.reference.namespace.as_deref() {
        Some("Entrez Gene") => gene.entrez_id.is_some_and(|id| id.to_string() == value),
        Some("Ensembl") => gene
            .ensembl_id
            .as_ref()
            .is_some_and(|id| id.to_string() == value),
        Some("HGNC") => hgnc_id(value) == Some(gene.hgnc_id) || gene.symbol.as_str() == value,
        _ => false,
    }
}

fn hgnc_id(value: &str) -> Option<biodata::HgncId> {
    if let Ok(id) = value.parse() {
        return Some(id);
    }
    let number = value.parse::<u32>().ok()?;
    if number.to_string() != value {
        return None;
    }
    biodata::HgncId::new(number).ok()
}
