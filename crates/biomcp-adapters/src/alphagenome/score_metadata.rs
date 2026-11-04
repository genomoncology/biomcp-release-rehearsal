//! Map native score metadata independently of complete tensor protocol admission.
//! NaN scores remain native content; optional nonzero_mean follows a finite metadata contract.
//! https://raw.githubusercontent.com/google-deepmind/alphagenome/55943112b83439e0c2512110f1ae607d4b5b492a/src/alphagenome/models/v1/variant_scoring_utils.py
use super::proto;
use crate::mapped::Problems;
use biomcp_fetch::BadReply;
use biomcp_types::{
    AlphaGenomeBiosample, AlphaGenomeGene, AlphaGenomeOntology, AlphaGenomeTrack,
    AlphaGenomeVariant,
};
use serde::de::{DeserializeOwned, value::StrDeserializer};
fn native<T: DeserializeOwned>(name: &str) -> Result<T, BadReply> {
    T::deserialize(StrDeserializer::<serde::de::value::Error>::new(name)).map_err(|_| BadReply)
}
pub(super) fn variant(value: &proto::Variant) -> Result<AlphaGenomeVariant, BadReply> {
    Ok(AlphaGenomeVariant {
        chromosome: value.chromosome.parse().map_err(|_| BadReply)?,
        position: value.position,
        reference_bases: value.reference_bases.parse().map_err(|_| BadReply)?,
        alternate_bases: value.alternate_bases.parse().map_err(|_| BadReply)?,
    })
}
pub(super) fn gene(
    value: proto::GeneScorerMetadata,
    path: &str,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Option<AlphaGenomeGene> {
    let field = format!("{path}.gene_id");
    let Some(_) = problems.optional(
        &field,
        Some(value.gene_id.parse::<biodata::EnsemblGeneId>()),
    ) else {
        *refusals = crate::Refusal::Refused;
        return None;
    };
    let Some(gene_id) = problems.optional(&field, Some(crate::mapped::text(&value.gene_id))) else {
        *refusals = crate::Refusal::Refused;
        return None;
    };
    Some(AlphaGenomeGene {
        gene_id,
        name: value.name,
        strand: problems.optional(
            &format!("{path}.strand"),
            value.strand.map(|value| {
                proto::Strand::try_from(value)
                    .map_err(|_| BadReply)
                    .and_then(|value| native(value.as_str_name()))
            }),
        ),
        kind: value.r#type,
        junction_start: value.junction_start,
        junction_end: value.junction_end,
    })
}
pub(super) fn track(
    value: proto::TrackMetadata,
    path: &str,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Option<AlphaGenomeTrack> {
    let Some(_) = problems.optional(
        &format!("{path}.name"),
        Some(crate::mapped::text(&value.name)),
    ) else {
        *refusals = crate::Refusal::Refused;
        return None;
    };
    let Some(strand) = problems.optional(
        &format!("{path}.strand"),
        Some(
            proto::Strand::try_from(value.strand)
                .map_err(|_| BadReply)
                .and_then(|value| native(value.as_str_name())),
        ),
    ) else {
        *refusals = crate::Refusal::Refused;
        return None;
    };
    let nonzero_mean = problems.optional(
        &format!("{path}.nonzero_mean"),
        value.nonzero_mean.map(|value| {
            if value.is_finite() {
                Ok(f64::from(value))
            } else {
                Err(BadReply)
            }
        }),
    );
    Some(AlphaGenomeTrack {
        name: value.name,
        strand,
        ontology_term: value
            .ontology_term
            .and_then(|value| ontology(value, path, problems, refusals)),
        biosample: value
            .biosample
            .map(|value| biosample(value, path, problems)),
        assay: value.assay,
        histone_mark_code: value.histone_mark_code,
        transcription_factor_code: value.transcription_factor_code,
        gtex_tissue: value.gtex_tissue,
        data_source: value.data_source,
        endedness: problems.optional(
            &format!("{path}.endedness"),
            value.endedness.map(|value| {
                proto::Endedness::try_from(value)
                    .map_err(|_| BadReply)
                    .and_then(|value| native(value.as_str_name()))
            }),
        ),
        genetically_modified: value.genetically_modified,
        nonzero_mean,
    })
}
fn ontology(
    value: proto::OntologyTerm,
    path: &str,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Option<AlphaGenomeOntology> {
    let Some(ontology_type) = problems.optional(
        &format!("{path}.ontology_term.ontology_type"),
        Some(
            proto::OntologyType::try_from(value.ontology_type)
                .map_err(|_| BadReply)
                .and_then(|value| native(value.as_str_name())),
        ),
    ) else {
        *refusals = crate::Refusal::Refused;
        return None;
    };
    Some(AlphaGenomeOntology {
        namespace: ontology_type,
        id: value.id,
    })
}
fn biosample(value: proto::Biosample, path: &str, problems: &mut Problems) -> AlphaGenomeBiosample {
    AlphaGenomeBiosample {
        kind: problems.optional(
            &format!("{path}.biosample.type"),
            Some(
                proto::BiosampleType::try_from(value.r#type)
                    .map_err(|_| BadReply)
                    .and_then(|value| native(value.as_str_name())),
            ),
        ),
        name: value.name,
        stage: value.stage,
    }
}
