//! Source coordinate metadata qualifies an unchanged native record key.
//! <https://biothings.io/blog/new-data-release-201606> describes root chrom/hg19/hg38.
//! Index selection alone never establishes a returned coordinate's assembly.
use super::raw::{Hit, SourceChromosome, SourceSpan};
use biodata::Build;

pub(super) fn source_build(hit: &Hit) -> Option<Build> {
    let (chromosome, expression) = hit.id.as_str().strip_prefix("chr")?.split_once(":g.")?;
    let SourceChromosome::Name(source_chromosome) = hit.chrom.as_ref()? else {
        return None;
    };
    if source_chromosome.as_str() != chromosome {
        return None;
    }
    let (position, _) = literal_span(expression)?;
    let span = (position, position);
    let mut found = None;
    for (build, value) in [(Build::GRCh37, &hit.hg19), (Build::GRCh38, &hit.hg38)] {
        if let Some(value) = value {
            let source = source_span(value)?;
            if source == span {
                if found.is_some() {
                    return None;
                }
                found = Some(build);
            }
        }
    }
    found
}
fn source_span(value: &SourceSpan) -> Option<(u64, u64)> {
    match value {
        SourceSpan::Bounds { start, end } if *start > 0 && end >= start => Some((*start, *end)),
        _ => None,
    }
}
// Compare only literal substitution positions. Other HGVS forms retain unknown assembly;
// no normalization, strand conversion, or allele transplant qualifies them.
fn literal_span(expression: &str) -> Option<(u64, &str)> {
    let length = expression.bytes().take_while(u8::is_ascii_digit).count();
    let position = expression.get(..length)?.parse::<u64>().ok()?;
    let (reference, alternate) = expression.get(length..)?.split_once('>')?;
    if position == 0
        || reference.len() != 1
        || alternate.len() != 1
        || !reference
            .bytes()
            .chain(alternate.bytes())
            .all(|b| b"ACGT".contains(&b))
    {
        return None;
    }
    Some((position, reference))
}

/// Source metadata alone qualifies the exact forward-reference SNV lookup.
pub(super) fn reference_locus(hit: &Hit) -> Option<(&str, Build, u64, &str)> {
    let build = source_build(hit)?;
    let (chromosome, expression) = hit.id.as_str().strip_prefix("chr")?.split_once(":g.")?;
    let (position, reference) = literal_span(expression)?;
    Some((chromosome, build, position, reference))
}
