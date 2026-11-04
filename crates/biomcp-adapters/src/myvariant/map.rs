//! Admit exact source records before mapping independently optional annotations.
use super::raw::{Gene, Hit};
use crate::{
    Mapped,
    mapped::Problems,
    raw::{Field, OneOrMany},
};
use biodata::{Build, Expression, ExpressionSyntax, Representation, Text, Variant, VariantId};
use biomcp_fetch::Reply;
use biomcp_types::{Source, SourceRecord, VariantHeader};
pub(super) fn candidates(
    hits: Vec<Field<Hit>>,
    query: &str,
    build: Option<Build>,
    allow_clinvar: bool,
    allow_snpeff: bool,
) -> Mapped<Vec<VariantHeader>> {
    let mut value = Vec::new();
    let mut refusals = crate::Refusal::None;
    let mut problems = Problems {
        source: Source::MyVariant,
        record: None,
        list: Vec::new(),
    };
    for raw in hits {
        problems.record = None;
        let Some(hit) = problems.field("identity", Some(raw)) else {
            refusals = crate::Refusal::Refused;
            continue;
        };
        problems.record = Some(hit.id.clone());
        if hit.check().is_err()
            || !hit.matches(query)
            || value.iter().any(|v: &VariantHeader| v.record.id == hit.id)
        {
            problems.required::<()>("identity", None);
            refusals = crate::Refusal::Refused;
            continue;
        }
        if let Some(row) = candidate(hit, build, allow_clinvar, allow_snpeff, &mut problems) {
            value.push(row);
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    Mapped {
        refusals,
        value,
        problems: problems.list,
    }
}
fn candidate(
    mut hit: Hit,
    build: Option<Build>,
    allow_clinvar: bool,
    allow_snpeff: bool,
    problems: &mut Problems,
) -> Option<VariantHeader> {
    let mut dbsnp = problems.field("dbsnp", hit.dbsnp.take());
    let aliases = problems.field("dbsnp.rsid", dbsnp.as_mut().and_then(|row| row.rsid.take()));
    let mut ids = Vec::new();
    for alias in aliases.into_iter().flat_map(OneOrMany::values) {
        let Some(alias) = problems.field("dbsnp.rsid", Some(alias)) else {
            continue;
        };
        let id = problems.optional("dbsnp.rsid", Some(alias.as_str().parse::<VariantId>()));
        if let Some(id) =
            problems.required("dbsnp.rsid", id.filter(|id| matches!(id, VariantId::Rs(_))))
        {
            ids.push(id);
        }
    }
    if allow_clinvar && let Some(cv) = problems.field("clinvar", hit.clinvar) {
        let id = problems.field("clinvar.variant_id", Some(cv.variant_id));
        if let Some(id) = problems.optional(
            "clinvar.variant_id",
            id.map(|id| {
                u32::try_from(id)
                    .map_err(|_| biodata::Error::ClinVarVariationIdInvalid)
                    .and_then(|id| {
                        biodata::ClinVarVariationId::new(id)
                            .map_err(|_| biodata::Error::ClinVarVariationIdInvalid)
                    })
            }),
        ) {
            ids.push(VariantId::ClinVarVariation(id));
        }
    }
    let mut component_terms = component_terms(&ids).into_vec();
    let transcripts = super::transcripts::map(
        hit.snpeff.filter(|_| allow_snpeff),
        &hit.id,
        &mut problems.list,
    );
    if !transcripts.is_empty() {
        component_terms.extend(super::transcripts::label());
    }
    let genes = genes(dbsnp.and_then(|row| row.gene), problems);
    let build = problems.required("representations.build", build);
    let expression = problems.optional(
        "identity",
        Some(Expression::new(
            ExpressionSyntax::HgvsG,
            hit.id.clone(),
            None,
        )),
    )?;
    Some(VariantHeader {
        identity: Variant {
            ids: ids.into(),
            representations: vec![Representation {
                source: Source::MyVariant.text(),
                expression,
                build,
                allele: None,
            }]
            .into(),
            genes: genes.into(),
        },
        source: Source::MyVariant,
        record: SourceRecord {
            namespace: biodata::IdentifierNamespace::Unknown,
            id: hit.id,
            url: None,
        },
        transcripts,
        myvariant: None,
        requested: None,
        sections: None,
        component_terms: component_terms.into(),
    })
}
fn component_terms(ids: &[VariantId]) -> Box<[biomcp_types::ComponentLabel]> {
    if ids
        .iter()
        .any(|id| matches!(id, VariantId::ClinVarVariation(_)))
    {
        Text::new("clinvar")
            .ok()
            .zip(super::components::label(super::components::CLINVAR.terms).ok())
            .map(|(component, terms)| biomcp_types::ComponentLabel { component, terms })
            .into_iter()
            .collect()
    } else {
        Box::default()
    }
}

fn genes(
    raw: Option<Field<OneOrMany<Field<Gene>>>>,
    problems: &mut Problems,
) -> Vec<biodata::SourceGeneRef> {
    let source = problems.field("dbsnp.gene", raw);
    source
        .into_iter()
        .flat_map(OneOrMany::values)
        .filter_map(|raw| {
            let gene = problems.field("dbsnp.gene", Some(raw))?;
            let symbol = problems.field("dbsnp.gene.symbol", gene.symbol)?;
            problems.optional(
                "dbsnp.gene.symbol",
                Some(biodata::SourceGeneRef::new(symbol, None, None, None)),
            )
        })
        .collect()
}
