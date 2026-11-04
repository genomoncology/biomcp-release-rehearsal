use super::{
    Resolution,
    raw::{Allele as RawAllele, Genomic},
};
use crate::Mapped;
use crate::mapped::problem;
use biodata::{Allele, Build, Expression, Representation, SequenceLocation, Span0, Text, Variant};

use biomcp_types::{Problem, Source, SourceRecord, VariantHeader};
pub(super) fn allele(raw: RawAllele, expected: Option<&str>) -> Resolution {
    if let Some(error) = &raw.error_type {
        return rejection(&raw, error);
    }
    let mut fields = crate::mapped::Problems {
        source: Source::AlleleRegistry,
        record: expected.and_then(crate::mapped::record),
        list: Vec::new(),
    };
    let Some((url, record, id)) = fields.optional("@id", Some(raw.identity(expected))) else {
        return Resolution::Refused(fields.list);
    };
    fields.record = Some(record.clone());
    let mut ids = vec![id];
    ids.extend(super::annotations::ids(raw.external, &mut fields));
    let mut problems = Vec::new();
    let mut representations = Vec::new();
    for g in fields.fields("genomicAlleles", raw.genomic) {
        genomic(g, &record, &mut representations, &mut problems);
    }
    representations.extend(super::annotations::forms(raw.transcripts, &mut fields));
    problems.extend(fields.list);
    let value = VariantHeader {
        requested: None,
        identity: Variant {
            ids: ids.into(),
            representations: representations.into(),
            genes: Box::default(),
        },
        source: Source::AlleleRegistry,
        record: SourceRecord {
            namespace: biodata::IdentifierNamespace::Unknown,
            id: record,
            url: Some(url),
        },
        transcripts: Box::default(),
        myvariant: None,
        sections: None,
        component_terms: Box::default(),
    };
    Resolution::Found(Box::new(Mapped {
        refusals: crate::Refusal::None,
        value,
        problems,
    }))
}
// Do not silently drop a source mismatch when its explanatory text cannot fit.
fn rejection(raw: &RawAllele, error: &Text) -> Resolution {
    let actual = raw
        .actual_allele
        .as_ref()
        .and_then(crate::raw::Field::as_ref)
        .filter(|actual| {
            error.as_str() == "IncorrectReferenceAllele"
                && !actual.as_str().is_empty()
                && actual.as_str().bytes().all(|base| b"ACGTN".contains(&base))
        });
    let Some(actual) = actual else {
        return Resolution::Rejected(error.clone());
    };
    match Text::new(&format!("{error}: expected reference {actual}")) {
        Ok(rejection) => Resolution::Rejected(rejection),
        Err(_) => Resolution::Refused(vec![problem(Source::AlleleRegistry, None, "actualAllele")]),
    }
}

fn genomic(g: Genomic, record: &Text, out: &mut Vec<Representation>, problems: &mut Vec<Problem>) {
    let mut fields = crate::mapped::Problems {
        source: Source::AlleleRegistry,
        record: Some(record.clone()),
        list: Vec::new(),
    };
    let assembly = fields.field("representations.build", g.assembly);
    let build = fields.optional(
        "representations.build",
        assembly.map(|a| a.as_str().parse::<Build>()),
    );
    let chromosome = fields.field("representations.chromosome", g.chromosome);
    let notations = fields.fields("genomicAlleles.hgvs", g.hgvs);
    let coordinates = fields.field("genomicAlleles.coordinates", g.coordinates);
    problems.extend(fields.list);
    for notation in notations {
        let expression = match Expression::hgvs(notation.clone()) {
            Ok(v) => v,
            Err(_) => {
                problems.push(problem(
                    Source::AlleleRegistry,
                    Some(record.clone()),
                    "representations.expression",
                ));
                continue;
            }
        };
        let sequence = notation
            .as_str()
            .parse::<biodata::HgvsText>()
            .and_then(|h| h.accession());
        let agrees = chromosome.as_ref().is_none_or(|given| {
            build
                .zip(sequence.as_ref().ok())
                .is_none_or(|(build, accession)| {
                    biodata::Chromosome::from_accession(*accession, build)
                        .is_ok_and(|chromosome| chromosome.name() == given.as_str())
                })
        });
        if !agrees {
            problems.push(problem(
                Source::AlleleRegistry,
                Some(record.clone()),
                "representations.chromosome",
            ));
        }
        let allele = if let Some(coordinates) = &coordinates
            && coordinates.len() == 1
        {
            coordinates.first().and_then(|c| {
                Allele::new(
                    SequenceLocation {
                        sequence: sequence.ok()?,
                        span: Span0::new(c.start, c.end).ok()?,
                    },
                    c.reference.parse().ok()?,
                    c.allele.parse().ok()?,
                )
                .ok()
            })
        } else {
            None
        };
        if allele.is_none() && coordinates.is_some() {
            problems.push(problem(
                Source::AlleleRegistry,
                Some(record.clone()),
                "representations.allele",
            ));
        }
        out.push(Representation {
            source: Source::AlleleRegistry.text(),
            expression,
            build,
            allele: allele.filter(|_| agrees),
        });
    }
}
