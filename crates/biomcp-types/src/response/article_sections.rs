use crate::{Capped, NcbiGeneId, Section, Text};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// Normalized annotation category in deterministic tie order.
    AnnotationKind, SourceUnknown {
        /// Gene mention.
        Gene => "gene",
        /// Genetic variant mention.
        Variant => "variant",
        /// Disease mention.
        Disease => "disease",
        /// Chemical mention.
        Chemical => "chemical",
    }
}
/// One normalized entity with its mention count.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AnnotatedEntity {
    /// Source concept type.
    pub kind: AnnotationKind,
    /// Source normalized name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Number of annotation mentions.
    pub count: u64,
    /// Gene identity, including a variant's associated gene.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrez_id: Option<NcbiGeneId>,
    /// Source MeSH identity for a disease or chemical.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<Text>,
    /// Source HGVS expression for a variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs: Option<Text>,
    /// Source reference SNP identity for a variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rsid: Option<Text>,
}
/// Capped entity annotations and the distinct total.
pub type Annotations = Capped<AnnotatedEntity>;
/// Landed article sections with independent source states.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleSections {
    /// Publisher PDF link; no file is downloaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf: Option<Section<crate::PdfLink>>,
    /// Index terms from the card's own EFetch citation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexing: Option<Section<crate::Indexing>>,
    /// JATS supplementary files confirmed by the bucket record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplements: Option<Section<crate::Supplements>>,
    /// Licensed PMC body outline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulltext: Option<Section<crate::FullText>>,
    /// Machine-written Semantic Scholar summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tldr: Option<Section<Tldr>>,
    /// Citation counts from NIH iCite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Section<Citations>>,
    /// Normalized PubTator3 entities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Section<Annotations>>,
}

/// Independently reported iCite counts; absent lists are never confirmed zeros.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Citations {
    /// Number of citing PubMed papers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cited_by: Option<u64>,
    /// Number of distinct clinical citing papers, when the list was present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cited_by_clinical: Option<u64>,
    /// Number of distinct references, when the list was present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<u64>,
    /// Relative citation ratio, only when confirmed non-provisional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rcr: Option<crate::FiniteNumber>,
}

/// Machine-written source summary with independently reported counts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Tldr {
    /// Complete one-sentence source summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    /// Source model version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<Text>,
    /// Explicitly identifies generated prose.
    pub machine_written: bool,
    /// Total source citation count, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cited_by: Option<u64>,
    /// Source influential citation count, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influential_cited_by: Option<u64>,
}
