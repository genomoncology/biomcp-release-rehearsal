use crate::{
    ComponentLabel, DiseaseGet, DiseaseOntology, DiseaseSearch, DiseaseSection, ResponseBody,
};
use biodata::{
    Attribution, Curie, DiseaseAssociation, DiseaseRelation, DiseaseStatistic, DiseaseSynonym, Text,
};
use serde::{Deserialize, Serialize};
/// A disease identity whose optional display label never determines record admission.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiseaseConcept {
    /// Source-stated exact identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Curie>,
    /// Native display label when supplied and valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pub label: Option<Text>,
}
/// One source candidate without a selected identity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiseaseHeader {
    /// Native concept and optional exact ID.
    pub concept: DiseaseConcept,
    /// The explicit queried ontology.
    pub ontology: DiseaseOntology,
    /// Publisher obsolete flag when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obsolete: Option<bool>,
    /// Source record and version.
    pub attribution: Attribution,
}
/// Exact identity and available pages without hidden enrichment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiseaseCard {
    /// Exact requested identifier.
    pub requested_id: Curie,
    /// Actual returned identifier.
    pub returned_id: Curie,
    /// Native relation proving a source-accepted alternate identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<DiseaseRelation>,
    /// Native identity and complete primary label.
    pub concept: DiseaseConcept,
    /// Complete source definition when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<Text>,
    /// Publisher obsolete flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obsolete: Option<bool>,
    /// Publisher alternatives, never an automatic replacement.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub consider: Option<Box<[Curie]>>,
    /// Publisher explicit replacement identifiers; null preserves no native replacement.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<Box<[Curie]>>,
    /// Publisher record and version.
    pub attribution: Attribution,
    /// Available named sections.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub sections: Box<[DiseaseSection]>,
    /// Applicable component terms.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[ComponentLabel]>,
}
impl ResponseBody for DiseaseCard {
    const SCHEMA: &'static str = "biomcp.disease-card/6";
    type Query = DiseaseGet;
}
/// One publisher catalog site, independent of ontology identities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiseaseSite {
    /// Native publisher site code.
    pub code: Text,
    /// Complete publisher site label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
    /// Publisher active-state flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}
/// One native association and the component terms governing that row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DiseaseAssociationItem {
    /// Complete neutral association, including requested and actual endpoints.
    pub association: DiseaseAssociation,
    /// Independently attributed annotation components for this row.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[ComponentLabel]>,
}

page!(
    disease,
    DiseasePage,
    DiseaseHeader,
    DiseaseSearch,
    "biomcp.disease-page/5",
    "One source-ordered disease candidate page."
);
page!(
    disease,
    DiseaseSynonyms,
    DiseaseSynonym,
    DiseaseGet,
    "biomcp.disease-synonyms/5",
    "One publisher synonym page with scope and language."
);
page!(
    disease,
    DiseaseXrefs,
    DiseaseRelation,
    DiseaseGet,
    "biomcp.disease-xrefs/5",
    "Native cross-reference predicates without equivalence promotion."
);
page!(
    disease,
    DiseaseHierarchy,
    DiseaseRelation,
    DiseaseGet,
    "biomcp.disease-hierarchy/5",
    "Native parent relations with both endpoints."
);
page!(
disease,
DiseaseGenes,
DiseaseAssociationItem,
DiseaseGet,
"biomcp.disease-genes/9",
"Native association rows, not a unique-gene count.",
[
        /// Original response retrieval Unix seconds, independent of local delivery.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub fetched: Option<u64>,
    ]
);
page!(
disease,
DiseasePhenotypes,
DiseaseAssociationItem,
DiseaseGet,
"biomcp.disease-phenotypes/9",
"Native phenotype association rows without diagnostic inference.",
[
        /// Original response retrieval Unix seconds, independent of local delivery.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub fetched: Option<u64>,
    ]
);
page!(
disease,
DiseaseSites,
DiseaseSite,
DiseaseSearch,
"biomcp.disease-sites/5",
"Publisher site catalog without an ontology-to-site mapping.",
[
        /// Original response retrieval time in Unix seconds, distinct from publisher version/date.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub fetched: Option<u64>,
    ]
);
page!(
disease,
DiseaseIncidence,
DiseaseStatistic,
DiseaseGet,
"biomcp.disease-incidence/7",
"Native incidence rows preserving requested and returned dimensions.",
[
        /// Original response retrieval time in Unix seconds, distinct from dataset publication.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub fetched: Option<u64>,
    ]
);
page!(
disease,
DiseaseSurvival,
DiseaseStatistic,
DiseaseGet,
"biomcp.disease-survival/7",
"Native annual survival rows with separate observed and modeled values.",
[
        /// Original response retrieval time in Unix seconds, distinct from dataset publication.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub fetched: Option<u64>,
    ]
);
