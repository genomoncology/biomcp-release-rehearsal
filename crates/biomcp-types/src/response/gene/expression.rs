use crate::{FiniteNumber, Text};
use serde::{Deserialize, Serialize};
/// HPA tissue RNA abundance in normalized transcripts per million.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TissueNtpm {
    /// Exact source tissue name.
    pub tissue: Text,
    /// Finite nonnegative nTPM.
    pub ntpm: biodata::WithUnit<RnaAbundance, RnaUnit>,
}
/// HPA cancer RNA abundance in protein-coding transcripts per million.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CancerPtpm {
    /// Exact source cancer tissue name.
    pub tissue: Text,
    /// Finite nonnegative pTPM.
    pub ptpm: biodata::WithUnit<RnaAbundance, RnaUnit>,
}
/// Gene-level HPA expression and subcellular annotations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GeneExpression {
    /// Source RNA tissue specificity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rna_tissue_specificity: Option<Text>,
    /// Source RNA tissue distribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rna_tissue_distribution: Option<Text>,
    /// Tissue-specific nTPM pairs.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub rna_tissue_ntpm: Box<[TissueNtpm]>,
    /// Source RNA cancer specificity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rna_cancer_specificity: Option<Text>,
    /// Cancer-specific pTPM pairs.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub rna_cancer_ptpm: Box<[CancerPtpm]>,
    /// Source main subcellular locations.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub subcellular_main: Box<[Text]>,
    /// Source additional subcellular locations.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub subcellular_additional: Box<[Text]>,
}

/// A finite nonnegative RNA abundance expressed in the field's stated unit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "f64", extend("minimum" = 0)))]
pub struct RnaAbundance(FiniteNumber);
impl TryFrom<f64> for RnaAbundance {
    type Error = biodata::Error;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value < 0.0 {
            return Err(biodata::Error::NumberNonFinite);
        }
        FiniteNumber::try_from(value)
            .map(Self)
            .map_err(|_| biodata::Error::NumberNonFinite)
    }
}
impl From<RnaAbundance> for f64 {
    fn from(value: RnaAbundance) -> Self {
        value.0.into()
    }
}

biodata::wire_enum! {
/// Publisher RNA abundance units.
RnaUnit {
    /// Normalized transcripts per million.
    Ntpm => "nTPM",
    /// Protein-coding transcripts per million.
    Ptpm => "pTPM",
}
}
