use crate::{Build, FiniteNumber, PopulationDataType, PopulationSex, Text, VariantId};
use serde::{Deserialize, Serialize};
/// Bounded population summary with independent optional sequencing datasets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantPopulation {
    /// Source allele identifier.
    pub variant: Text,
    /// Source dataset selector.
    pub dataset: Text,
    /// Source dataset assembly.
    pub build: Build,
    /// Release shared by independently checked exome and genome totals in this response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dataset_version: Option<Text>,
    /// Exome observation; omitted when the source dataset is null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exome: Option<Box<GnomadCounts>>,
    /// Genome observation; omitted when the source dataset is null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genome: Option<Box<GnomadCounts>>,
    /// Independent source joint totals, or an explicit empty state when none were supplied.
    #[serde(default = "joint_empty")]
    pub joint: crate::Section<Box<GnomadCounts>>,
}
fn joint_empty() -> crate::Section<Box<GnomadCounts>> {
    crate::Section::Empty {
        source: crate::Source::Gnomad,
        record: None,
    }
}
/// Native gnomAD counts retained when a dataset release cannot be established.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GnomadCounts {
    /// Checked total from a release supplied alongside matching source counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<crate::PopulationFrequency>,
    /// Source sequencing sample set.
    pub data_type: PopulationDataType,
    /// Complete native group ID, including its source sex suffix when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_id: Option<Text>,
    /// Source ancestry code decoded only by the documented source grammar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ancestry_group: Option<Text>,
    /// Source sex stratum decoded only by the documented source grammar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sex: Option<PopulationSex>,
    /// Native allele count, including reported zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ac: Option<u64>,
    /// Native allele denominator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub an: Option<u64>,
    /// Source-published total homozygote count, never calculated from other counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homozygote_count: Option<u64>,
    /// Native source filters, never converted to a classification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<Box<[Text]>>,
}
/// One page of exact rsID catalog associations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantGwas {
    /// Exact source rsID.
    pub rs_id: VariantId,
    /// Native source associations, without inferred clinical meaning.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub associations: Box<[GwasAssociation]>,
    /// Exact source page bounds and total.
    pub page: crate::PageInfo,
    /// Continue this section using the same rsID and source selectors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<crate::VariantGet>,
}
/// Source association with unchanged trait and effect labels.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasAssociation {
    /// Native association identity.
    pub id: biodata::GwasAssociationId,
    /// Study accession.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub study: Option<Text>,
    /// Source reported traits.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub traits: Box<[Text]>,
    /// Source mapped trait identifiers and whole labels, without clinical inference.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub ontology_traits: Box<[GwasTrait]>,
    /// Every source effect allele paired with its rsID.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub alleles: Box<[GwasEffectAllele]>,
    /// Reported p value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p_value: Option<FiniteNumber>,
    /// Authoritative exact source probability, including double underflow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_p_value: Option<biodata::ScientificPValue>,
    /// Source author attribution, without local publication inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Text>,
    /// Original native mapped gene context; reported genes remain in loci.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub mapped_genes: Box<[Text]>,
    /// Native mapping location strings; no assumed assembly or coordinate conversion.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub locations: Box<[Text]>,
    /// Source lower effect interval bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_lower: Option<FiniteNumber>,
    /// Source upper effect interval bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci_upper: Option<FiniteNumber>,
    /// Source beta text, including units and direction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beta: Option<Text>,
    /// Source odds ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub odds_ratio: Option<FiniteNumber>,
    /// Source allele frequency text, without interpreting NR.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_frequency: Option<Text>,
    /// Source effect interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<Text>,
    /// Source publication identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication: Option<Text>,
}
/// Catalog effect allele paired with its exact rsID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct GwasEffectAllele {
    /// Source rsID.
    pub rs_id: Text,
    /// Source allele text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allele: Option<Text>,
}
/// Population summary and one explicitly requested ancestry-detail page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantPopulationResult {
    /// Source population state; missing datasets stay absent within its summary.
    pub population: crate::Section<VariantPopulation>,
    /// Selected native groups, qualified by sequencing subset.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub groups: Box<[GnomadCounts]>,
    /// Source allele candidates when this input does not identify one joinable allele.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub candidates: Box<[crate::VariantHeader]>,
}
impl super::ResponseBody for VariantPopulationResult {
    const SCHEMA: &'static str = "biomcp.variant-population/13";
    type Query = crate::VariantPopulationQuery;
}

/// Source ontology mapping as supplied by GWAS Catalog.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(extend("x-identifier" = true)))]
pub struct GwasTrait {
    /// Native authority; unknown means the source did not state one.
    pub namespace: biodata::IdentifierNamespace,
    /// Exact source ontology identifier spelling.
    pub id: Text,
    /// Whole source trait label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
}

impl GwasTrait {
    /// Retain the full source ID and expose only the authority stated by its checked grammar.
    pub fn new(id: Text, label: Option<Text>) -> Self {
        let namespace = id
            .as_str()
            .parse::<biodata::GwasTraitId>()
            .ok()
            .and_then(|value| Text::new(value.namespace()).ok())
            .into();
        Self {
            namespace,
            id,
            label,
        }
    }
}
