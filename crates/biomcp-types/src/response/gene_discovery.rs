use biodata::{
    Gene, GeneLocation, GeneStatus, GeneSymbol, HgncId, HgncStatus, LocusGroup, MatchKind, Text,
};
use serde::{Deserialize, Serialize};
/// A source-stated match, preserving the text that matched.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Match {
    /// The match category.
    pub kind: MatchKind,
    /// The source text that matched the input.
    pub text: Text,
}
/// Enough gene identity to choose a search result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GeneHeader {
    /// HGNC identifier.
    pub hgnc_id: HgncId,
    /// Source-preserved symbol.
    pub symbol: GeneSymbol,
    /// HGNC locus group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locus_group: Option<LocusGroup>,
    /// Source-stated location, including unparsed publisher text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<GeneLocation>,
    /// HGNC record status.
    pub status: GeneStatus,
    /// Gene name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Match information when this header answers an input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#match: Option<Match>,
}
/// Resolution outcomes preserve ambiguity and withdrawn identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "resolution", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum GeneResolution {
    /// One resolved identity and its source match.
    Resolved {
        /// The whole gene entity.
        gene: Box<Gene>,
        /// How the source matched the input.
        r#match: Match,
    },
    /// Every candidate, with no silent selection.
    Ambiguous {
        /// Ordered candidates supplied by the resolver.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        candidates: Box<[GeneHeader]>,
    },
    /// The original withdrawn identity and stated successors.
    Withdrawn {
        /// Original HGNC identifier.
        hgnc_id: HgncId,
        /// Original symbol.
        symbol: GeneSymbol,
        /// Stated HGNC withdrawn-file status.
        hgnc_status: HgncStatus,
        /// All stated merge destinations.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        merged_into: Box<[HgncId]>,
    },
    /// No source identity matched.
    NotFound {},
}
