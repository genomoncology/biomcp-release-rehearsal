use crate::{GwasAssociation, GwasGet, GwasSearch, GwasTrait, PageInfo, ResponseBody, Text};
use biodata::{GwasAssociationId, GwasStudyId, Pmid};
use serde::{Deserialize, Serialize};
/// One native association page, also used for a study associations section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasPage {
    /// Shared complete association records.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[GwasAssociation]>,
    /// Native page scope and total.
    pub page: PageInfo,
    /// Complete qualifying count, unknown until the threshold walk reaches its end.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifying_total: Option<u64>,
    /// Same-selector search continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "GwasPageQuery"))]
    pub continuation: Option<GwasPageQuery>,
}
impl ResponseBody for GwasPage {
    const SCHEMA: &'static str = "biomcp.gwas-page/8";
    type Query = GwasPageQuery;
}
/// Exact association card; no second association representation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasAssociationCard {
    /// The shared source association record.
    pub association: GwasAssociation,
}
impl ResponseBody for GwasAssociationCard {
    const SCHEMA: &'static str = "biomcp.gwas-association/6";
    type Query = GwasGet;
}
/// Source reported gene identity, without a causal-gene interpretation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasReportedGene {
    /// Native source gene name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Exact source Ensembl identifiers.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub ensembl_ids: Box<[Text]>,
    /// Exact source Entrez identifiers.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub entrez_ids: Box<[Text]>,
}
/// One native locus with independent reported genes and risk-allele text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasLocus {
    /// Complete source locus description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Text>,
    /// Source strongest-risk-allele strings.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub strongest_risk_alleles: Box<[Text]>,
    /// Reported source genes; never inferred from names.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub author_reported_genes: Box<[GwasReportedGene]>,
    /// Native locus self link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub source_url: Option<Text>,
}
/// One bounded locus section for an association.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasLoci {
    /// Exact owning association.
    pub association: GwasAssociationId,
    /// Native locus rows.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[GwasLocus]>,
    /// Same-resource relation continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "GwasGet"))]
    pub continuation: Option<GwasGet>,
}
impl ResponseBody for GwasLoci {
    const SCHEMA: &'static str = "biomcp.gwas-loci/4";
    type Query = GwasGet;
}
/// Native study card; independent source descriptions are never reconciled.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasStudy {
    /// Exact accession.
    pub id: GwasStudyId,
    /// Source publication identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication: Option<Pmid>,
    /// Native reported disease traits.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub reported_traits: Box<[Text]>,
    /// Every native mapped ontology label and ID.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub ontology_traits: Box<[GwasTrait]>,
    /// Whole discovery sample prose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_sample_description: Option<Text>,
    /// Whole replication sample prose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replication_sample_description: Option<Text>,
    /// Native discovery ancestry summaries.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub discovery_ancestries: Box<[Text]>,
    /// Native replication ancestry summaries.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub replication_ancestries: Box<[Text]>,
    /// Source technologies.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub genotyping_technologies: Box<[Text]>,
    /// Source SNP count; missing differs from zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snp_count: Option<u64>,
    /// Native platform description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Text>,
    /// Native array manufacturer labels.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub array_manufacturers: Option<Box<[Text]>>,
    /// Native gene-environment interaction flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gxe: Option<bool>,
    /// Native gene-gene interaction flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gxg: Option<bool>,
    /// Native imputation flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imputed: Option<bool>,
    /// Native pooled study flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pooled: Option<bool>,
    /// Native user-requested study flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_requested: Option<bool>,
    /// Source availability flag; grants no download or reuse permission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_statistics_available: Option<bool>,
    /// Native summary statistics link; not a local download.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub summary_statistics_url: Option<Text>,
    /// Native terms link without an inferred reuse grant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub license_url: Option<Text>,
}
impl ResponseBody for GwasStudy {
    const SCHEMA: &'static str = "biomcp.gwas-study/6";
    type Query = GwasGet;
}
/// Native country and region metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasCountry {
    /// Original country label including NR.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country: Option<Text>,
    /// Native source major area when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub major_area: Option<Text>,
    /// Original region label when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Text>,
}
/// One independent source ancestry row; counts are never summed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasAncestry {
    /// Native discovery or replication stage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<Text>,
    /// Source row count; no unique-person denominator is inferred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_of_individuals: Option<u64>,
    /// Original ancestry group labels.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub ancestry_groups: Box<[Text]>,
    /// Native origin metadata.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub country_of_origin: Box<[GwasCountry]>,
    /// Native recruitment metadata.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub country_of_recruitment: Box<[GwasCountry]>,
    /// Native row self link, checked against its owning study by the adapter.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub source_url: Text,
}
/// Bounded retained ancestry relation for one exact study.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasAncestries {
    /// Exact owning study.
    pub study: GwasStudyId,
    /// Independent source rows.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[GwasAncestry]>,
    /// Retained relation continuation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "GwasGet"))]
    pub continuation: Option<GwasGet>,
}
impl ResponseBody for GwasAncestries {
    const SCHEMA: &'static str = "biomcp.gwas-ancestries/5";
    type Query = GwasGet;
}

/// Search and study-association pages echo their original checked request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", content = "query", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum GwasPageQuery {
    /// Native association search.
    Search(GwasSearch),
    /// Exact study associations relation.
    Get(GwasGet),
}

/// One publication's native study page, without inferred clinical meaning.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasStudyPage {
    /// Source study records and their native association totals.
    pub items: Box<[GwasStudyRow]>,
    /// Native study page scope and total.
    pub page: PageInfo,
    /// Same publication and study result mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<GwasSearch>,
}
impl ResponseBody for GwasStudyPage {
    const SCHEMA: &'static str = "biomcp.gwas-study-page/1";
    type Query = GwasSearch;
}
/// Source study metadata and the count from its association page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasStudyRow {
    /// The same source metadata returned by an exact study lookup.
    #[serde(flatten)]
    pub study: GwasStudy,
    /// Catalog association total; absent when the count request fails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub association_count: Option<u64>,
}
