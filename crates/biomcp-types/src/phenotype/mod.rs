mod query;
use crate::{ComponentLabel, DiseaseAssociationItem, ResponseBody};
use biodata::{
    PhenotypeCard, PhenotypeHeader, PhenotypeHierarchyLink, PhenotypeSynonym, PhenotypeXref,
};
pub use query::{PhenotypeGet, PhenotypeSearch, PhenotypeSection, PhenotypeSource};
use serde::{Deserialize, Serialize};
/// Independent source identity card and its available sections.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PhenotypeTermCard {
    /// Complete source-specific native card.
    pub term: PhenotypeCard,
    /// Sections available on this source record.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub sections: Box<[PhenotypeSection]>,
    /// Applicable publisher component terms.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[ComponentLabel]>,
}
impl ResponseBody for PhenotypeTermCard {
    const SCHEMA: &'static str = "biomcp.phenotype-card/5";
    type Query = PhenotypeGet;
}

page!(
    phenotype,
    PhenotypePage,
    PhenotypeHeader,
    PhenotypeSearch,
    "biomcp.phenotype-search/5",
    "Native HP search header page, without local ranking."
);
page!(
    phenotype,
    PhenotypeSynonyms,
    PhenotypeSynonym,
    PhenotypeGet,
    "biomcp.phenotype-synonyms/4",
    "Source synonym rows preserving publisher classes."
);
page!(
    phenotype,
    PhenotypeXrefs,
    PhenotypeXref,
    PhenotypeGet,
    "biomcp.phenotype-xrefs/4",
    "Source reference rows without equivalence promotion."
);
page!(
    phenotype,
    PhenotypeHierarchy,
    PhenotypeHierarchyLink,
    PhenotypeGet,
    "biomcp.phenotype-hierarchy/5",
    "Direct source hierarchy links without graph traversal."
);
macro_rules! associations {
    ($name:ident, $schema:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub struct $name {
            /// Native association rows in source order, without deduplication.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub items: Box<[DiseaseAssociationItem]>,
            /// Publisher direct graph scope; this supplies no causal assertion.
            pub direct: bool,
            /// Applicable publisher component terms.
            #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
            pub components: Box<[ComponentLabel]>,
            /// Original source retrieval time, distinct from dataset release.
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub fetched: Option<u64>,
        }
        impl ResponseBody for $name {
            const SCHEMA: &'static str = $schema;
            type Query = PhenotypeGet;
        }
    };
}
associations!(
    PhenotypeDiseases,
    "biomcp.phenotype-diseases/7",
    "Direct source disease association rows without clinical inference."
);
associations!(
    PhenotypeGenes,
    "biomcp.phenotype-genes/7",
    "Direct source human gene association rows with native taxon."
);
