use super::raw::Pos;
use crate::mapped::record as record_text;
use crate::mapped::{problem, push_problem};
use biodata::{Build, GenomicInterval, PartialDate, Span0, Strand, Summary, SummarySource, Text};

pub(super) fn summary(value: String) -> Result<Summary, biodata::Error> {
    let parsed = value.strip_suffix("].").and_then(|text| {
        let (body, marker) = text.rsplit_once(" [provided by RefSeq, ")?;
        let (month, year) = marker.split_once(' ')?;
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let month = months.iter().position(|candidate| *candidate == month)? as u8 + 1;
        if year.len() != 4 || !year.bytes().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        let date = PartialDate::new(year.parse().ok()?, Some(month), None).ok()?;
        Some((body, date))
    });
    let (body, dated) = parsed.map_or((value.as_str(), None), |(body, date)| (body, Some(date)));
    Ok(Summary {
        text: Text::from_source(body)?,
        source: SummarySource::RefSeq,
        dated,
    })
}

pub(super) fn interval(
    value: Pos,
    build: Build,
    id: NcbiGeneId,
    field: &str,
    problems: &mut Vec<Problem>,
) -> Option<GenomicInterval> {
    let start = value.start?.value()?;
    let end = value.end?.value()?;
    if start == 0 || end < start {
        return None;
    }
    let strand = match value.strand {
        None => None,
        Some(Field::Value(1)) => Some(Strand::Plus),
        Some(Field::Value(-1)) => Some(Strand::Minus),
        Some(_) => {
            push_problem(
                problems,
                Source::MyGene,
                record_text(id),
                &format!("{field}.strand"),
            );
            None
        }
    };
    // Source-chain inference: the pinned MyGene parser copies Ensembl bounds.
    // Ensembl counts closed one-based positions; equal endpoints span one base.
    // https://github.com/biothings/mygene.info/blob/7515239530e9784f311623bd925c4ae3ba284415/src/hub/dataload/sources/ensembl/parser.py#L224
    // https://beta.ensembl.org/help/articles/accessing-ensembl-s-refget-services
    Some(GenomicInterval {
        build: Some(build),
        chromosome: crate::raw::chromosome(&value.chr?.value()?)?,
        span: Span0::new(start - 1, end).ok()?,
        strand,
    })
}

use super::{
    GeneExtras,
    raw::{Genes, Hit},
};
use crate::{
    Mapped,
    raw::{Field, OneOrMany},
};
use biodata::{HgncId, NcbiGeneId};

use biomcp_types::{Problem, Source};
use std::collections::BTreeMap;

pub(super) fn extras(
    reply: Genes,
    ids: &[NcbiGeneId],
    problems: &mut Vec<Problem>,
    refusals: &mut crate::Refusal,
) -> Vec<Mapped<GeneExtras>> {
    let mut hits: BTreeMap<NcbiGeneId, Vec<Hit>> = BTreeMap::new();
    for hit in reply.0 {
        match hit.query.parse::<NcbiGeneId>() {
            Ok(id) if ids.contains(&id) => hits.entry(id).or_default().push(hit),
            _ => {
                *refusals = crate::Refusal::Refused;
                problems.push(problem(Source::MyGene, None, "query"));
            }
        }
    }
    for id in ids {
        if !hits.contains_key(id) {
            problems.push(problem(Source::MyGene, record_text(*id), "query"));
        }
    }
    hits.into_iter()
        .filter_map(|(id, mut hits)| {
            if hits.len() != 1 {
                *refusals = crate::Refusal::Refused;
                problems.push(problem(Source::MyGene, record_text(id), "query"));
                return None;
            }
            map_hit(hits.pop()?, id, problems, refusals)
        })
        .collect()
}

fn map_hit(
    hit: Hit,
    id: NcbiGeneId,
    problems: &mut Vec<Problem>,
    refusals: &mut crate::Refusal,
) -> Option<Mapped<GeneExtras>> {
    if hit.notfound == Some(true) {
        return None;
    }
    let returned = hit
        .entrezgene
        .and_then(Field::value)
        .and_then(|value| value.parse::<NcbiGeneId>().ok());
    if returned != Some(id) {
        *refusals = crate::Refusal::Refused;
        problems.push(problem(Source::MyGene, record_text(id), "entrez_id"));
        return None;
    }
    let hgnc_present = hit.hgnc.is_some();
    let hgnc = hit
        .hgnc
        .and_then(Field::value)
        .and_then(|value| format!("HGNC:{value}").parse::<HgncId>().ok());
    let mut mapped = Mapped {
        refusals: crate::Refusal::None,
        value: GeneExtras {
            query_id: id,
            entrez_id: returned?,
            hgnc_id: hgnc,
            summary: None,
            spans: Box::new([]),
        },
        problems: Vec::new(),
    };
    if hgnc_present && hgnc.is_none() {
        mapped
            .problems
            .push(problem(Source::MyGene, record_text(id), "hgnc_id"));
    }
    if let Some(field) = hit.summary {
        match field.value().map(summary) {
            Some(Ok(summary)) => mapped.value.summary = Some(summary),
            _ => mapped
                .problems
                .push(problem(Source::MyGene, record_text(id), "summary")),
        }
    }
    let mut spans = Vec::new();
    positions(
        hit.genomic_pos,
        Build::GRCh38,
        id,
        &mut spans,
        &mut mapped.problems,
    );
    positions(
        hit.genomic_pos_hg19,
        Build::GRCh37,
        id,
        &mut spans,
        &mut mapped.problems,
    );
    mapped.value.spans = spans.into_boxed_slice();
    Some(mapped)
}

fn positions(
    field: Option<Field<OneOrMany<Field<Pos>>>>,
    build: Build,
    id: NcbiGeneId,
    spans: &mut Vec<GenomicInterval>,
    problems: &mut Vec<Problem>,
) {
    let Some(field) = field else {
        return;
    };
    let name = match build {
        Build::GRCh38 => "genomic_pos",
        _ => "genomic_pos_hg19",
    };
    match field.value() {
        None => problems.push(problem(Source::MyGene, record_text(id), name)),
        Some(values) => {
            for value in values.values() {
                match value
                    .value()
                    .and_then(|value| interval(value, build, id, name, problems))
                {
                    Some(value) => spans.push(value),
                    None => problems.push(problem(Source::MyGene, record_text(id), name)),
                }
            }
        }
    }
}
