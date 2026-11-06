use crate::{Source, VariantGet, VariantNamespace};
use serde::{Deserialize, Serialize};

/// Variant request with any assembly default that changed the source request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantGetQuery {
    /// Checked input and selectors, including the chosen assembly.
    #[serde(flatten)]
    query: crate::VariantGetEcho,
    /// True when BioMCP selected the echoed assembly for MyVariant.
    #[serde(skip_serializing_if = "Option::is_none")]
    build_defaulted: Option<bool>,
}
impl VariantGetQuery {
    /// Preserve selectors, disclose the assembly default and name only the identity resolver that answered.
    pub fn new(query: VariantGet, source: Source, outcome: Option<&crate::SourceOutcome>) -> Self {
        let defaulted = source == Source::MyVariant
            && query.build().is_none()
            && matches!(
                query.input().namespace(),
                VariantNamespace::Rs | VariantNamespace::ClinVar
            );
        Self {
            query: if defaulted {
                query.with_myvariant_build()
            } else {
                query
            }
            .answered_query(outcome),
            build_defaulted: defaulted.then_some(true),
        }
    }
}
