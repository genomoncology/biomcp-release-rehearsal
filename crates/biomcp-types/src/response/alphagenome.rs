//! Preserve paged native AlphaGenome values without ranking or clinical interpretation.
use super::alphagenome_enums::{
    AlphaGenomeBiosampleType, AlphaGenomeEndedness, AlphaGenomeOntologyType, AlphaGenomeStrand,
};
use serde::{Deserialize, Serialize};
/// Native ontology identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(extend("x-identifier" = true)))]
pub struct AlphaGenomeOntology {
    /// Publisher ontology vocabulary.
    pub namespace: AlphaGenomeOntologyType,
    /// Publisher numeric identifier.
    pub id: i64,
}
/// Native biosample metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AlphaGenomeBiosample {
    /// Publisher sample category.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<AlphaGenomeBiosampleType>,
    /// Publisher sample name.
    pub name: String,
    /// Publisher stage text, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
}
/// Native metadata for one tensor track.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AlphaGenomeTrack {
    /// Publisher track name.
    pub name: String,
    /// Publisher strand value.
    pub strand: AlphaGenomeStrand,
    /// Native ontology identity, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ontology_term: Option<AlphaGenomeOntology>,
    /// Native biosample, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biosample: Option<AlphaGenomeBiosample>,
    /// Publisher assay text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assay: Option<String>,
    /// Publisher histone mark.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histone_mark_code: Option<String>,
    /// Publisher transcription factor code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription_factor_code: Option<String>,
    /// Publisher GTEx tissue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gtex_tissue: Option<String>,
    /// Publisher data source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_source: Option<String>,
    /// Publisher sequence endedness.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endedness: Option<AlphaGenomeEndedness>,
    /// Publisher modification flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genetically_modified: Option<bool>,
    /// Publisher nonzero mean, without assigning score units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonzero_mean: Option<f64>,
}
/// Native gene metadata for a tensor row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AlphaGenomeGene {
    /// Publisher gene identifier.
    pub gene_id: crate::Text,
    /// Publisher gene name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Publisher gene strand.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strand: Option<AlphaGenomeStrand>,
    /// Publisher gene type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Publisher junction start, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub junction_start: Option<i64>,
    /// Publisher junction end, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub junction_end: Option<i64>,
}
/// Exact genomic coordinates supplied in the request and echoed by the source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AlphaGenomeVariant {
    /// Exact chromosome sent.
    pub chromosome: biodata::Contig,
    /// One-based source position.
    pub position: i64,
    /// Exact reference sequence sent.
    pub reference_bases: biodata::Sequence,
    /// Exact alternate sequence sent.
    pub alternate_bases: biodata::Sequence,
}
/// A requested interval; the scoring response does not echo an interval field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AlphaGenomeInterval {
    /// Exact chromosome sent.
    pub chromosome: biodata::Contig,
    /// Zero-based inclusive start.
    pub start: i64,
    /// Zero-based exclusive end.
    pub end: i64,
    /// Requested strand.
    pub strand: AlphaGenomeStrand,
}
biodata::wire_enum! {
    /// The scorer explicitly requested at this ordinal; not a native response label.
    AlphaGenomeScorer {
        /// Gene-mask log fold change requested on RNA_SEQ.
        GeneMaskLfcRnaSeq => "gene-mask-lfc-rna-seq",
        /// Gene-mask splicing requested on SPLICE_SITES, with width absent.
        GeneMaskSplicingSpliceSites => "gene-mask-splicing-splice-sites",
        /// Center-mask DIFF_MEAN requested on DNASE, width 501.
        CenterMaskDiffMeanDnase501 => "center-mask-diff-mean-dnase-501",
    }
}
biodata::wire_enum! {
    /// Original tensor scalar representation.
    AlphaGenomeDataType {
        /// Publisher FLOAT32 tensor storage.
        Float32 => "DATA_TYPE_FLOAT32",
    }
}
/// One native tensor value with its unchanged coordinate and metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AlphaGenomeValue {
    /// Native row-major tensor coordinate; plane, row and track indices.
    pub tensor_index: [u32; 3],
    /// Exact numeric value represented by the source scalar.
    pub value: super::alphagenome_scalar::AlphaGenomeScalar,
    /// Native metadata for the corresponding track.
    pub track: AlphaGenomeTrack,
    /// Native gene metadata, omitted for a nongene aggregate row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene: Option<AlphaGenomeGene>,
}
/// Independent page of one requested scorer's native values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AlphaGenomeScorerPage {
    /// Request position used to associate this unlabelled response group.
    pub request_ordinal: u32,
    /// Requested scorer identity, distinct from native metadata.
    pub requested_scorer: AlphaGenomeScorer,
    /// Original scalar representation.
    pub data_type: AlphaGenomeDataType,
    /// Complete native tensor dimensions, including every plane.
    pub shape: [u32; 3],
    /// Paged values in native row-major order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub rows: Box<[AlphaGenomeValue]>,
    /// Value paging within this scorer only.
    pub page: crate::PageInfo,
    /// Next explicit section request, when more native values remain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<crate::VariantGet>,
}
/// Paged native scorer results with explicit request provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantAlphaGenome {
    /// Source-resolved assembly used for the request.
    pub build: biodata::Build,
    /// Exact request identity, checked against native metadata.
    pub variant: AlphaGenomeVariant,
    /// Exact requested interval, never claimed as a returned interval.
    pub requested_interval: AlphaGenomeInterval,
    /// Checkpoint field sent; empty means the publisher default was requested.
    pub requested_model_version: String,
    /// Native scorer groups associated only by requested position.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub scorers: Box<[AlphaGenomeScorerPage]>,
}
