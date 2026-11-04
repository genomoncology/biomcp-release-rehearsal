mod civic;
mod clingen;
mod gencc;
pub use gencc::{GenccGene, GenccSubmission};
mod constraint;
mod expression;
mod oncokb;
mod pathways;
mod protein;
use super::{ItemStatus, ResponseBody, Section};
use crate::GeneSection;
use crate::{GeneGet, GeneHeader, GeneInput, GeneResolution, GeneSearch};
pub use civic::CivicGene;
pub use clingen::{ClinGenGene, ClinGenValidity};
pub use constraint::Constraint;
pub use expression::{CancerPtpm, GeneExpression, RnaAbundance, RnaUnit, TissueNtpm};
pub use oncokb::{Oncokb, OncokbCard, OncokbGeneType, OncokbTranscript};
pub use pathways::{Capped, PathwayRef, ReactomeId};
pub use protein::Protein;
use serde::{Deserialize, Serialize};
/// Answer to one requested input.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GeneItem {
    /// Verified keyed source card annotation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oncokb: Option<OncokbCard>,
    /// Original checked input.
    pub input: GeneInput,
    /// This input's outcome.
    pub status: ItemStatus,
    /// Absent only when identity lookup failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<GeneResolution>,
    /// Requested sections, present only for a resolved identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<GeneSections>,
    /// Applicable sections left out of this request.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub more: Box<[GeneSection]>,
}
/// Gene answers in input order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GeneCards {
    /// Requested inputs and their answers.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[GeneItem]>,
}
/// One page of gene headers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GenePage {
    /// Headers after ranking and paging.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[GeneHeader]>,
}
impl ResponseBody for GeneCards {
    const SCHEMA: &'static str = "biomcp.gene-cards/13";
    type Query = GeneGet;
}
impl ResponseBody for GenePage {
    const SCHEMA: &'static str = "biomcp.gene-page/8";
    type Query = GeneSearch;
}
/// Explicit source synchronization selector.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SyncQuery {
    /// Source whose complete identity data is synchronized.
    pub source: crate::Source,
}
/// Published bundle metadata after synchronization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SyncResult {
    /// Catalog content revision and acquisition time, separate from source headers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<crate::StudySnapshot>,
    /// Native publisher ETag when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etag: Option<Box<str>>,
    /// Native publisher Last-Modified when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<Box<str>>,
    /// Native catalog rows examined before identity admission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_rows: Option<u64>,
    /// Imported rows.
    pub rows: u64,
    /// Imported source-file bytes before native file decoding.
    pub bytes: u64,
    /// Source body bytes received in this sync; excludes previous partials, 304 reuse and replay.
    pub downloaded_bytes: u64,
    /// Distinct repeated identities across the whole source version report.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duplicate_identities: Option<u64>,
}
impl ResponseBody for SyncResult {
    const SCHEMA: &'static str = "biomcp.sync/7";
    type Query = SyncQuery;
}

/// Landed gene sections, each retaining its source and explicit state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GeneSections {
    /// Reviewed human proteins listed by HGNC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protein: Option<Section<Box<[Protein]>>>,
    /// CIViC gene-level curation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub civic: Option<Section<CivicGene>>,
    /// ClinGen validity and dosage assessments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clingen: Option<Section<ClinGenGene>>,
    /// GenCC assessments, retained separately for each submitter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gencc: Option<Section<GenccGene>>,
    /// Population constraint reported by gnomAD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<Section<Constraint>>,
    /// Human Protein Atlas expression annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<Section<GeneExpression>>,
    /// Lowest-level human Reactome mappings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pathways: Option<Section<Capped<PathwayRef>>>,
    /// Keyed source gene details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oncokb: Option<Section<Oncokb>>,
}
