//! Keyed source requests bind every accepted response to its complete explicit context.
use super::SPEC;
use crate::raw::Field;
use biodata::Build;
use biomcp_fetch::Input;
use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, Reply, RequestError};
use serde::Deserialize;

const ANNOTATION: Endpoint = SPEC
    .get("/api/v1/annotate/mutations/byHGVSg")
    .fresh(Freshness::Days(1));

/// Native source response; clinical labels have not been mapped or interpreted.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    /// Source echo, required before admission to the shared reply cache.
    pub(super) query: Query,
    /// Provider exact variant curation flag.
    pub(super) variant_exist: Option<Field<bool>>,
    /// Native gene curation flag.
    pub(super) gene_exist: Option<Field<bool>>,
    /// Native investigated-but-unknown flag, without deriving a classification.
    #[serde(rename = "vus", alias = "VUS")]
    pub(super) vus: Option<Field<bool>>,
    /// Native statistical hotspot flag.
    pub(super) hotspot: Option<Field<bool>>,
    /// Provider reports invalid inputs as a 200 with validation errors.
    pub(super) errors: Option<Vec<ValidationError>>,
    /// Provider alternate allele curation flag.
    pub(super) allele_exist: Option<Field<bool>>,
    /// Native oncogenicity label.
    pub(super) oncogenic: Option<Field<String>>,
    /// Native mutation effect.
    pub(super) mutation_effect: Option<Field<MutationEffect>>,
    /// Native source text.
    pub(super) gene_summary: Option<Field<String>>,
    /// Native source text.
    pub(super) variant_summary: Option<Field<String>>,
    /// Native source text.
    pub(super) tumor_type_summary: Option<Field<String>>,
    /// Native source text.
    pub(super) diagnostic_summary: Option<Field<String>>,
    /// Native source text.
    pub(super) prognostic_summary: Option<Field<String>>,
    /// Native source implications in source order.
    pub(super) treatments: Option<Field<Vec<Field<Treatment>>>>,
    /// Native diagnostic implications in source order.
    pub(super) diagnostic_implications: Option<Field<Vec<Field<Implication>>>>,
    /// Native prognostic implications in source order.
    pub(super) prognostic_implications: Option<Field<Vec<Field<Implication>>>>,
    /// Source release.
    pub(super) data_version: Option<Field<String>>,
    /// Source release date, without an invented conversion.
    pub(super) last_update: Option<Field<String>>,
}
/// Complete source-echoed request identity.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    /// Source genomic query.
    pub(super) hgvs: biodata::Text,
    /// Source assembly.
    pub(super) reference_genome: Build,
    /// Source tumor context.
    pub(super) tumor_type: String,
    /// Source-supplied gene context, never inferred locally.
    pub(super) hugo_symbol: Option<Field<biodata::GeneSymbol>>,
    /// Source-supplied protein alteration, never converted to a genomic allele.
    pub(super) alteration: Option<Field<biodata::Text>>,
}
/// Native mutation effect and text, without local interpretation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationEffect {
    /// Native label.
    pub(super) known_effect: Option<Field<String>>,
    /// Native text.
    pub(super) description: Option<Field<String>>,
    /// Native effect citations.
    pub(super) citations: Option<Field<Citations>>,
}
/// Native publication citations.
#[derive(Debug, Deserialize)]
pub struct Citations {
    /// Source PMID identities.
    pub(super) pmids: Option<Field<Vec<Field<String>>>>,
    /// Native non-PubMed citations.
    pub(super) abstracts: Option<Field<Vec<Field<Abstract>>>>,
}
/// Native drug identity.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Drug {
    /// Source label.
    pub(super) drug_name: Option<Field<String>>,
    /// Native NCIt code, when supplied.
    pub(super) ncit_code: Option<Field<String>>,
}
/// Native tumor concept.
#[derive(Debug, Deserialize)]
pub struct TumorType {
    /// OncoTree code.
    pub(super) code: Option<Field<String>>,
    /// Native display label.
    pub(super) name: Option<Field<String>>,
}
/// Native treatment row, including the provider's alteration grouping.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Treatment {
    /// Native source level.
    pub(super) level: Option<Field<biodata::Text>>,
    /// Source tumor context.
    pub(super) level_associated_cancer_type: Option<Field<TumorType>>,
    /// Source drugs in native order.
    pub(super) drugs: Option<Field<Vec<Field<Drug>>>>,
    /// Source alteration group, never replaced with a genomic subject.
    pub(super) alterations: Option<Field<Vec<Field<String>>>>,
    /// Native citations.
    pub(super) pmids: Option<Field<Vec<Field<String>>>>,
    /// Native source description.
    pub(super) description: Option<Field<String>>,
    /// Native excluded tumor contexts.
    pub(super) level_excluded_cancer_types: Option<Field<Vec<Field<TumorType>>>>,
    /// Native approved indication labels.
    pub(super) approved_indications: Option<Field<Vec<Field<String>>>>,
    /// Native FDA level label.
    pub(super) fda_level: Option<Field<String>>,
    /// Native non-PubMed citations.
    pub(super) abstracts: Option<Field<Vec<Field<Abstract>>>>,
}
/// Native diagnostic or prognostic implication.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Implication {
    /// Native source level.
    pub(super) level_of_evidence: Option<Field<biodata::Text>>,
    /// Native source tumor concept.
    pub(super) tumor_type: Option<Field<TumorType>>,
    /// Source alteration group.
    pub(super) alterations: Option<Field<Vec<Field<String>>>>,
    /// Native citations.
    pub(super) pmids: Option<Field<Vec<Field<String>>>>,
    /// Native source description.
    pub(super) description: Option<Field<String>>,
    /// Native non-PubMed citations.
    pub(super) abstracts: Option<Field<Vec<Field<Abstract>>>>,
}
impl Reply for Annotation {
    fn check(&self) -> Result<(), BadReply> {
        if self.errors.as_ref().is_none_or(Vec::is_empty) {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}
/// Read only the licensed endpoint under the existing required-key source policy.
/// A different source query, assembly or tumor context can never enter the reply cache.
pub fn annotation(
    leg: &Leg<'_>,
    hgvsg: &str,
    build: Build,
    tumor_type: &str,
) -> Result<Annotation, RequestError> {
    let assembly = build.to_string();
    leg.call(
        &ANNOTATION,
        Input::Params(&[
            ("hgvsg", Arg::Text(hgvsg)),
            ("referenceGenome", Arg::Text(&assembly)),
            ("tumorType", Arg::Text(tumor_type)),
        ]),
        biomcp_fetch::Read::json().check(&|row: &Annotation| {
            if row.query.hgvs.as_str() == hgvsg
                && row.query.reference_genome == build
                && row.query.tumor_type == tumor_type
            {
                Ok(())
            } else {
                Err(BadReply)
            }
        }),
    )
    .map(|received| (received.value, received.meta))
    .map(|(row, _)| row)
}

/// Native citation text and link, kept without claiming reuse rights.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Abstract {
    /// Native citation content.
    pub(super) abstract_content: Option<Field<String>>,
    /// Source citation link.
    pub(super) link: Option<Field<biodata::Text>>,
}
/// Provider validation failures must not enter the annotation cache.
#[derive(Debug, Deserialize)]
pub struct ValidationError {
    /// Source failure code.
    #[serde(rename = "type")]
    pub kind: String,
    /// Source message, never interpreted as a clinical classification.
    pub message: String,
}
