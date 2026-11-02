//! Exact source namespace joins; display names never participate in identity.
use crate::{Ctx, status};
use biodata::{PathwayMember, UniProtAcc, UniProtIsoform};
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
    let accessions: Vec<_> = rows.iter().filter_map(accession).collect();
    let keys: Vec<_> = accessions
        .iter()
        .map(|id| format!("uniprot:{id}"))
        .collect();
    let names: Vec<_> = keys.iter().map(String::as_str).collect();
    let mut genes = Vec::new();
    if !names.is_empty() {
        let leg = cx.fetch.leg(&hgnc::SPEC, deadline).fresh(cx.fresh);
        let mapped = hgnc::lookup(&leg, Find::AnyExact(KeyKind::Xref, &names));
        sources.push(status::source(
            Source::Hgnc,
            leg.finish(),
            mapped.as_ref().err().copied(),
        ));
        if let Ok(mapped) = mapped {
            genes = mapped.value;
            problems.extend(mapped.problems);
        }
    }
    for row in rows {
        let matched = accession(row).and_then(|id| {
            let mut matching = genes.iter().filter(|g| g.uniprot.contains(&id));
            matching
                .next()
                .filter(|_| matching.next().is_none())
                .map(|g| g.hgnc_id)
        });
        row.gene = matched;
        if matched.is_none() {
            problems.push(Problem {
                source: Source::Reactome,
                record: format!(
                    "{}:{}",
                    row.reference.namespace.as_deref().unwrap_or(""),
                    row.reference.id
                )
                .parse()
                .ok(),
                field: "genes.gene".into(),
                code: ProblemCode::ValueUnknown,
            });
        }
    }
}
fn accession(row: &PathwayMember) -> Option<UniProtAcc> {
    if row.reference.namespace.as_deref() != Some("uniprot") {
        return None;
    }
    let stated = UniProtIsoform::new(row.reference.id.as_str())?;
    let base = stated
        .as_str()
        .split_once('-')
        .map_or(stated.as_str(), |(base, _)| base);
    UniProtAcc::new(base).ok()
}
