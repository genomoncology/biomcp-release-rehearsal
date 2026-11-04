use crate::{GeneSymbol, PageInfo, Text, VariantGet};
use serde::{Deserialize, Serialize};

/// Source residue recurrence counts without allele-frequency or clinical interpretation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantHotspots {
    /// Source rows remain candidates when no transcript/consequence join exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<crate::Reason>,
    /// Native residue records in source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub records: Box<[HotspotRecord]>,
    /// Disclosure bounds for this section.
    pub page: PageInfo,
    /// Continue this section without changing identity selectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<VariantGet>,
}

/// Counts for one source transcript and residue label.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HotspotRecord {
    /// Whether transcript and consequence were linked by the identity source.
    pub membership: HotspotMembership,
    /// Gene as stated by the source.
    pub gene: GeneSymbol,
    /// Native transcript spelling, without an inferred accession version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Text>,
    /// Native residue label; no genomic allele is inferred.
    pub residue: Text,
    /// Tumors reported for this residue by the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tumor_count: Option<u64>,
    /// Source alternate amino acids and their counts.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub alternates: Box<[HotspotAlternate]>,
}

/// One source amino-acid alternate count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HotspotAlternate {
    /// Source alternate amino-acid spelling.
    pub amino_acid: Text,
    /// Source count, without deriving a frequency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
}

biodata::wire_enum! {
    /// Source transcript and consequence join state.
    HotspotMembership {
        /// Provider-qualified transcript and consequence matched.
        Exact => "exact",
        /// A native source row with no established transcript/consequence join.
        Candidate => "candidate",
    }
}
