use super::QueryInvalid;
use biodata::{PathwayId, Text};
use serde::Deserialize;
mod get;
mod relation;
mod search;
pub use get::PathwayGet;
pub use relation::{PathwayEventsQuery, PathwayGenesQuery};
pub use search::PathwaySearch;
biodata::wire_enum! {
    #[derive(Default)]
    /// One explicit pathway source, without fallback or cross-source title joins.
    PathwaySource {
        #[default]
        #[doc = "Human Reactome database records."] Reactome => "reactome",
        #[doc = "Human WikiPathways metadata catalog."] Wikipathways => "wikipathways"
    }
}
biodata::wire_enum! {
    described
    /// Supported source-owned pathway disclosures.
    PathwaySection {
        #[doc = "Complete native source description."] Summary => "summary",
        #[doc = "Namespaced native participants with exact resolved gene joins."] Genes => "genes",
        #[doc = "Native Reactome event objects and unresolved database references."] Events => "events",
        #[doc = "Each source-supported section's bounded first page."] All => "all"
    }
}
fn default_limit() -> u32 {
    5
}
fn valid_limit(limit: u32) -> bool {
    (1..=50).contains(&limit)
}
fn reactome(id: &PathwayId) -> bool {
    matches!(id, PathwayId::Reactome(_))
}
// CLI requests carry one checked native spelling; neutral records remain source-tagged.
mod native_id {
    use super::{Deserialize, PathwayId, Text};
    pub(super) fn serialize<S: serde::Serializer>(
        id: &PathwayId,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_str(id)
    }
    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<PathwayId, D::Error> {
        let v = Text::deserialize(d)?;
        v.as_str().parse().map_err(serde::de::Error::custom)
    }
}

impl PathwaySource {
    /// Native source identity for the selected provider.
    pub const fn source(self) -> crate::Source {
        match self {
            Self::Reactome => crate::Source::Reactome,
            Self::Wikipathways => crate::Source::WikiPathways,
        }
    }
}

impl PathwaySection {
    /// Publishers implementing this disclosure.
    pub const fn sources(self) -> &'static [crate::Source] {
        use crate::Source;
        match self {
            Self::Events => &[Source::Reactome],
            Self::Summary | Self::Genes | Self::All => &[Source::Reactome, Source::WikiPathways],
        }
    }
}
