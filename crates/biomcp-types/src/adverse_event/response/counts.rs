//! Source-native reaction buckets without rates, denominators or causal inference.
use biodata::Text;
use serde::{Deserialize, Serialize};

/// One exact reaction term and its openFDA report count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventReactionCount {
    /// Exact native MedDRA term, without normalization or inferred drug association.
    pub term: Text,
    /// Native count of matching reports containing this term, not exposed people.
    pub count: u64,
}

/// Bounded native count response; buckets may overlap and are never summed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AdverseEventReactionCounts {
    /// Native reaction buckets in provider order; a report may contribute to several.
    pub items: Box<[AdverseEventReactionCount]>,
    /// The provider returned as many terms as requested; more terms may exist.
    /// This does not establish a total or prove that terms were omitted.
    pub limit_reached: bool,
}
