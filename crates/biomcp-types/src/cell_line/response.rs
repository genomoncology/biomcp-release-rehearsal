use super::{CellLineAssayCountQuery, CellLineGet, CellLineInput, CellLineSearch, CellLineSection};
use crate::{Capped, ItemStatus, ResponseBody, Section};
use biodata::{
    CellLineEntryDates, CellLineName, CellLineSex, CellLineVariation, CellLineWarning,
    CellLineXref, CellosaurusAccession, ChemblCellId, PartialDate, Text,
};
use serde::{Deserialize, Serialize};
/// Compact source identity with independently capped species and diseases.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineHeader {
    /// Exact primary accession.
    pub accession: CellosaurusAccession,
    /// Recommended display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Native category.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<Text>,
    /// Up to three complete species references and their native total.
    pub species: Capped<CellLineXref>,
    /// Up to three complete disease references and their independent total.
    pub diseases: Capped<CellLineXref>,
    /// Up to three of this line's variations that show every filter term, and their total. Only variation-filtered searches carry it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variations: Option<Capped<CellLineVariation>>,
}
/// Source-ordered cell line search headers; totals remain unknown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLinePage {
    /// Native result order, without reranking or selected identity.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[CellLineHeader]>,
    /// What a variation filter matched; only variation-filtered searches carry it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variation_match: Option<CellLineVariationMatch>,
    /// Independently fetched publisher release, never a result total.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original retrieval seconds; not a source release date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
}
/// The source terms a variation filter sent, and what a match on them means.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineVariationMatch {
    /// Terms sent to the source, in its spelling.
    pub terms: Box<[Text]>,
    /// How far a match on those terms can be trusted.
    pub note: Box<str>,
}
impl CellLineVariationMatch {
    /// The one statement every variation-filtered reply carries.
    pub(super) fn new(terms: Vec<Text>) -> Self {
        Self {
            terms: terms.into(),
            note: "Matched on Cellosaurus variation text as the source writes it, so a short or empty \
                result is not proof of absence. Terms are not tied to one variation: a line can \
                match on separate variations, \
                and a gene term also matches lines marked None_reported. Each row's variations lists \
                only variations showing every term; a row with none matched across variations.".into(),
        }
    }
}
/// One requested cell identity with explicit ambiguity and requested section states.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineCard {
    /// Exact original caller identity.
    pub input: CellLineInput,
    /// Resolved, absent, ambiguous or failed input outcome.
    pub status: ItemStatus,
    /// Selected verified identity only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<CellLineHeader>,
    /// Candidate identities without first-hit selection.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub candidates: Box<[CellLineHeader]>,
    /// Selected entry dates and version, separate from release metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dates: Option<CellLineEntryDates>,
    /// Native age expression when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<Text>,
    /// Native sex label when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<CellLineSex>,
    /// Selected card warnings, up to three whole rows; not a fetch state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Capped<CellLineWarning>>,
    /// Publisher URL for the selected identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Option<Text>,
    /// Only explicitly requested sections carry a state.
    pub sections: CellLineSections,
    /// Supported disclosures not requested.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub more: Box<[CellLineSection]>,
    /// Independently fetched publisher release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original retrieval seconds, distinct from entry and release dates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
}
/// Requested sections carry explicit states; absent members were not requested.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineSections {
    /// Requested names section state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Section<CellLineNames>>,
    /// Requested identity section state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Section<CellLineIdentityPage>>,
    /// Requested xrefs section state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xrefs: Option<Section<CellLineXrefs>>,
    /// Requested variants section state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Section<CellLineVariants>>,
    /// Requested chembl section state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chembl: Option<Section<CellLineChembl>>,
}
/// Native display names with collection-local totals and offset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineNames {
    /// Selected exact primary accession.
    pub accession: CellosaurusAccession,
    /// Whole source rows and the total in this collection only.
    pub items: Capped<CellLineName>,
    /// Offset within this collection, never within another section.
    pub offset: u32,
    /// Independently fetched publisher release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original complete-card retrieval seconds, distinct from source release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
}
/// Native source cross-references with collection-local totals and offset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineXrefs {
    /// Selected exact primary accession.
    pub accession: CellosaurusAccession,
    /// Whole source rows and the total in this collection only.
    pub items: Capped<CellLineXref>,
    /// Offset within this collection, never within another section.
    pub offset: u32,
    /// Independently fetched publisher release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original complete-card retrieval seconds, distinct from source release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
}
/// Whole native variation rows with collection-local totals and offset.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineVariants {
    /// Selected exact primary accession.
    pub accession: CellosaurusAccession,
    /// Whole source rows and the total in this collection only.
    pub items: Capped<CellLineVariation>,
    /// Offset within this collection, never within another section.
    pub offset: u32,
    /// Independently fetched publisher release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original complete-card retrieval seconds, distinct from source release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
}
/// Species, diseases and warnings page independently; each total belongs to its own collection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineIdentityPage {
    /// Selected exact primary accession.
    pub accession: CellosaurusAccession,
    /// Whole species rows and their source total.
    pub species: Capped<CellLineXref>,
    /// Whole disease rows and their independent source total.
    pub diseases: Capped<CellLineXref>,
    /// Whole scoped warning rows and their independent source total.
    pub warnings: Capped<CellLineWarning>,
    /// Requested offset applied independently to each collection.
    pub offset: u32,
    /// Independently fetched publisher release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original complete-card retrieval seconds, distinct from source release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,
}
/// One exact accession-linked ChEMBL cell identity; no assay request is implied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineChemblRecord {
    /// Native cell ChEMBL ID, never a compound or target.
    pub id: ChemblCellId,
    /// Independent native integer cell record identity.
    pub cell_id: biodata::Identifier<u64>,
    /// Native exact accession; adapter verifies the join.
    pub cellosaurus_id: CellosaurusAccession,
    /// Native display name without name-based joining.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Native description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Text>,
    /// Native organism label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organism: Option<Text>,
    /// Native organism taxon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxon: Option<u32>,
    /// Native tissue label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tissue: Option<Text>,
    /// Native CLO identifier; omission is source absence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clo: Option<Text>,
    /// Native EFO identifier; omission is source absence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub efo: Option<Text>,
}
/// Exact accession-linked metadata with its own native collection total.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineChembl {
    /// Independent ChEMBL release, distinct from retrieval time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original retrieval seconds; never a source release date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,

    /// Requested exact accession.
    pub accession: CellosaurusAccession,
    /// Every selected whole metadata row and the native total.
    pub items: Capped<CellLineChemblRecord>,
    /// Native metadata page offset.
    pub offset: u32,
}
/// Native assay total for one source-local cell identity; no activity or sensitivity count.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineAssayCount {
    /// Independent ChEMBL release, distinct from retrieval time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<CellLineRelease>,
    /// Original retrieval seconds; never a source release date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fetched: Option<u64>,

    /// Exact requested ChEMBL cell ID.
    pub id: ChemblCellId,
    /// Native assay page total_count, including a reported zero.
    pub assay_count: u64,
}
impl ResponseBody for CellLinePage {
    const SCHEMA: &'static str = "biomcp.cell-line-page/7";
    type Query = CellLineSearch;
}
impl ResponseBody for CellLineCard {
    const SCHEMA: &'static str = "biomcp.cell-line-card/8";
    type Query = CellLineGet;
}
impl ResponseBody for CellLineNames {
    const SCHEMA: &'static str = "biomcp.cell-line-names/5";
    type Query = CellLineGet;
}
impl ResponseBody for CellLineIdentityPage {
    const SCHEMA: &'static str = "biomcp.cell-line-identity/6";
    type Query = CellLineGet;
}
impl ResponseBody for CellLineXrefs {
    const SCHEMA: &'static str = "biomcp.cell-line-xrefs/6";
    type Query = CellLineGet;
}
impl ResponseBody for CellLineVariants {
    const SCHEMA: &'static str = "biomcp.cell-line-variants/7";
    type Query = CellLineGet;
}
impl ResponseBody for CellLineChembl {
    const SCHEMA: &'static str = "biomcp.cell-line-chembl/7";
    type Query = CellLineGet;
}
impl ResponseBody for CellLineAssayCount {
    const SCHEMA: &'static str = "biomcp.cell-line-assay-count/6";
    type Query = CellLineAssayCountQuery;
}

/// Publisher release identity, fetched independently from record data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineRelease {
    /// Native release version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
    /// Native release update date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<PartialDate>,
}

/// Native filtered cell-line headers; the literal-search reply contract remains unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CellLineFilteredPage(pub CellLinePage);
impl ResponseBody for CellLineFilteredPage {
    const SCHEMA: &'static str = "biomcp.cell-line-filtered-page/1";
    type Query = super::CellLineSearchCommand;
}
