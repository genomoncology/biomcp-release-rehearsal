use super::{PgxAnnotationGet, PgxGet, PgxSearch, PgxSource, PgxTotalKind};
use crate::ResponseBody;
use biodata::{
    ClinPgxAccession, GeneSymbol, PgxAnnotationDetail, PgxAnnotationHeader, PgxAnnotationReference,
    PgxDiplotypeResult, PgxDrugRecommendation, PgxGeneIdentity, PgxGuidelineReference, PgxPair,
    PgxRecommendation,
};
use serde::{Deserialize, Serialize};
/// Source and declared gene context; no chosen pair or patient result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgxHeader {
    /// Explicit source.
    pub source: PgxSource,
    /// Declared source symbol.
    pub gene: GeneSymbol,
}
/// Independent collection scope; observed counts never imply native source totals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgxPageInfo {
    /// Offset in this collection.
    pub offset: u32,
    /// Number of whole returned rows.
    pub returned: u32,
    /// Exact source total or received bounded collection length.
    pub total: u64,
    /// Provenance of total.
    pub total_kind: PgxTotalKind,
}
macro_rules! collection {
    ($name:ident,$row:ty,$query:ty,$schema:literal) => {
        /// Independent source-ordered page of complete native records.
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name {
            /// Source and selected gene.
            pub header: PgxHeader,
            /// Complete native rows without duplicate summaries.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub items: Box<[$row]>,
            /// Meaning and bounds of this collection.
            pub page: PgxPageInfo,
        }
        impl ResponseBody for $name {
            const SCHEMA: &'static str = $schema;
            type Query = $query;
        }
    };
}
collection!(PgxPage, PgxPair, PgxSearch, "biomcp.pgx-page/6");
collection!(
    PgxGuidelines,
    PgxGuidelineReference,
    PgxGet,
    "biomcp.pgx-guidelines/4"
);
/// Independent recommendation contracts preserve native table and view shapes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "representation", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PgxRecommendations {
    /// Gene-keyed recommendation_view with native names and no inferred drug ID.
    View {
        /// Selected gene and source.
        header: PgxHeader,
        /// Complete view rows in source order.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        items: Box<[PgxRecommendation]>,
        /// Native source collection scope.
        page: PgxPageInfo,
    },
    /// Exact drug selector uses the distinct native recommendation table.
    Table {
        /// Selected gene and source.
        header: PgxHeader,
        /// Complete table rows without view-name enrichment.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        items: Box<[PgxDrugRecommendation]>,
        /// Native source collection scope.
        page: PgxPageInfo,
    },
}
impl ResponseBody for PgxRecommendations {
    const SCHEMA: &'static str = "biomcp.pgx-recommendations/5";
    type Query = PgxGet;
}
/// Compact exact source gene identity, not a selected gene-drug pair.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgxCard {
    /// Selected gene and source.
    pub header: PgxHeader,
    /// Native record version is not a dataset release.
    pub identity: PgxGeneIdentity,
}
impl ResponseBody for PgxCard {
    const SCHEMA: &'static str = "biomcp.pgx-card/4";
    type Query = PgxGet;
}
/// Every source-returned mapping for the exact declared expression.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgxDiplotype {
    /// Explicit gene and source.
    pub header: PgxHeader,
    /// Zero, one or multiple native mappings; never a selected patient phenotype.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub diplotypes: Box<[PgxDiplotypeResult]>,
}
impl ResponseBody for PgxDiplotype {
    const SCHEMA: &'static str = "biomcp.pgx-diplotype/4";
    type Query = PgxGet;
}
/// One independent ClinPGx collection, preserving its native min-tier shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "annotation_kind", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum PgxAnnotations {
    /// Complete clinical min rows including location and relationship arrays.
    Clinical {
        /// Declared gene and source.
        header: PgxHeader,
        /// Complete clinical min records.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        items: Box<[PgxAnnotationHeader]>,
        /// Observed bounded collection scope.
        page: PgxPageInfo,
    },
    /// Guideline references without invented full guidance.
    Guideline {
        /// Declared gene and source.
        header: PgxHeader,
        /// Complete guideline min references.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        items: Box<[PgxAnnotationReference]>,
        /// Observed bounded collection scope.
        page: PgxPageInfo,
    },
    /// Label references without invented full guidance.
    Label {
        /// Declared gene and source.
        header: PgxHeader,
        /// Complete label min references.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        items: Box<[PgxAnnotationReference]>,
        /// Observed bounded collection scope.
        page: PgxPageInfo,
    },
}
impl ResponseBody for PgxAnnotations {
    const SCHEMA: &'static str = "biomcp.pgx-annotations/5";
    type Query = PgxGet;
}
/// Exact clinical annotation identity, distinct from any genomic allele identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgxAnnotationIdentity {
    /// Explicit ClinPGx source.
    pub source: PgxSource,
    /// Selected exact accession.
    pub id: ClinPgxAccession,
}
/// Complete proved base-tier annotation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PgxAnnotation {
    /// Exact selected source identity.
    pub header: PgxAnnotationIdentity,
    /// Complete native base record, including history and genotype text.
    pub annotation: PgxAnnotationDetail,
}
impl ResponseBody for PgxAnnotation {
    const SCHEMA: &'static str = "biomcp.pgx-annotation/5";
    type Query = PgxAnnotationGet;
}
