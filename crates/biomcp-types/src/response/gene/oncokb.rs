use crate::Text;
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// OncoKB's stated gene classification.
    OncokbGeneType, SourceUnknown {
        /// Both stated roles.
        OncogeneAndTsg => "ONCOGENE_AND_TSG",
        /// Source-stated oncogene.
        Oncogene => "ONCOGENE",
        /// Source-stated tumor suppressor.
        Tsg => "TSG",
        /// Source states insufficient evidence.
        InsufficientEvidence => "INSUFFICIENT_EVIDENCE",
        /// Source states neither role.
        Neither => "NEITHER",
    }
}
/// Short source-stated annotation on a gene card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OncokbCard {
    /// Exact native classification, including an unpublished future literal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gene_type: Option<biodata::SourceText>,
    /// Coding for the five published source literals only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gene_type_code: Option<OncokbGeneType>,
    /// Highest sensitive level, as stated by the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_sensitive_level: Option<Text>,
    /// Highest resistance level, as stated by the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highest_resistance_level: Option<Text>,
    /// Data release version, when verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
}
/// Source transcript annotation for one genome build.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OncokbTranscript {
    /// Source-stated isoform, kept separate from HGNC MANE.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isoform: Option<Text>,
    /// Source-stated RefSeq transcript.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refseq: Option<Text>,
}
/// Whole OncoKB gene prose and independent transcript annotations.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Oncokb {
    /// Verified data release version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
    /// Whole source summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Text>,
    /// Whole source background.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Text>,
    /// GRCh37 source transcripts.
    pub grch37: OncokbTranscript,
    /// GRCh38 source transcripts.
    pub grch38: OncokbTranscript,
}
