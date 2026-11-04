//! Keep admitted transcript identities when independent genomic representations are invalid.
use super::{Resolution, raw::Response};
use crate::mapped::Problems;
use biodata::{Build, Expression, Representation, Variant};
use biomcp_types::{Source, SourceRecord, VariantHeader};

pub(super) fn resolve(raw: Response, input: &str) -> Resolution {
    let mut problems = Problems {
        source: Source::VariantValidator,
        record: None,
        list: Vec::new(),
    };
    let rejected = raw.flag.as_str() == "warning";
    let mut candidates = Vec::new();
    let mut refusals = crate::Refusal::None;
    if !rejected {
        for (_, row) in raw.records(input, &mut problems, &mut refusals) {
            let Some(key) = problems.text("identity", Some(&row.hgvs_transcript_variant)) else {
                refusals = crate::Refusal::Refused;
                continue;
            };
            let Some(expression) =
                problems.optional("identity", Some(Expression::hgvs(key.clone())))
            else {
                refusals = crate::Refusal::Refused;
                continue;
            };
            let mut representations = vec![Representation {
                source: Source::VariantValidator.text(),
                expression,
                build: None,
                allele: None,
            }];
            if let Some(loci) = problems.field("primary_assembly_loci", row.primary_assembly_loci) {
                for (assembly, raw) in loci {
                    let field = format!("primary_assembly_loci.{assembly}");
                    let Some(locus) = problems.field(&field, Some(raw)) else {
                        continue;
                    };
                    let Some(value) = problems.field(&field, Some(locus.hgvs_genomic_description))
                    else {
                        continue;
                    };
                    let Some(expression) = problems.optional(&field, Some(Expression::hgvs(value)))
                    else {
                        continue;
                    };
                    let build = match assembly.as_str() {
                        "grch37" | "hg19" => Some(Build::GRCh37),
                        "grch38" | "hg38" => Some(Build::GRCh38),
                        _ => None,
                    };
                    representations.push(Representation {
                        source: Source::VariantValidator.text(),
                        expression,
                        build,
                        allele: None,
                    });
                }
            }
            candidates.push(VariantHeader {
                identity: Variant {
                    ids: Box::default(),
                    representations: representations.into(),
                    genes: Box::default(),
                },
                source: Source::VariantValidator,
                record: SourceRecord {
                    namespace: biodata::IdentifierNamespace::Unknown,
                    id: key,
                    url: None,
                },
                transcripts: Box::default(),
                myvariant: None,
                requested: None,
                sections: None,
                component_terms: Box::default(),
            });
        }
    }
    Resolution {
        refusals,
        rejected,
        candidates,
        problems: problems.list,
    }
}
