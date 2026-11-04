use crate::{FiniteNumber, PredictorScore, Text};
use serde::{Deserialize, Serialize};
/// Metadata for a component carried by an aggregator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantComponent {
    /// Independently reviewed publisher terms; no blanket aggregator grant.
    pub terms: ComponentTerms,
    /// Component name from aggregator metadata.
    pub name: Text,
    /// Component version; omitted when the source reports none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<Text>,
    /// Component license URL, without any inferred grant.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<Text>,
    /// Aggregator build version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregator_build: Option<Text>,
    /// Aggregator build timestamp, distinct from a component release.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregator_built: Option<crate::SourceDateTimeValue>,
    /// Component upload timestamp when the source states one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploaded: Option<crate::SourceDateTimeValue>,
    /// Source-named component upload timestamps when the publisher supplies a map.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub uploads: Box<[ComponentUpload]>,
    /// Whole source snapshot warning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<Text>,
}
/// Source prediction scores and the terms of their individual components.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantPredictors {
    /// How the source chromosome label was reference-qualified.
    pub subject_qualification: SubjectQualification,
    /// Original context arrays, without inferred cross-field alignment.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub context: Box<[PredictorContext]>,
    /// Every retained source score in declared field order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub scores: Box<[PredictorScore]>,
    /// Independent component versions and licenses.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[VariantComponent]>,
}
/// Source conservation measurements; no local pathogenicity inference.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantConservation {
    /// How the source chromosome label was reference-qualified.
    pub subject_qualification: SubjectQualification,
    /// Original context arrays, without inferred cross-field alignment.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub context: Box<[PredictorContext]>,
    /// Every retained source measurement in declared field order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub scores: Box<[PredictorScore]>,
    /// Independent component versions and licenses.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[VariantComponent]>,
}
/// Historical COSMIC fields carried by MyVariant, without direct COSMIC access.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantCosmic {
    /// Number of native historical fragments before paging.
    pub total: u64,
    /// Typed next page for the same identity and section selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<crate::VariantGet>,
    /// Native historical records in source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub records: Box<[CosmicRecord]>,
    /// Snapshot version, dates, warning and terms.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub components: Box<[VariantComponent]>,
}
/// One source record from MyVariant's historical COSMIC copy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CosmicRecord {
    /// MyVariant parent record association; this does not establish allele equivalence.
    pub myvariant_record: Text,
    /// Fragment identity has not been independently verified.
    pub identity: CosmicIdentity,
    /// Native COSMIC record identifier.
    pub cosmic_id: biodata::CosmicId,
    /// Unmodified source reference spelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<Text>,
    /// Unmodified source alternate spelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alternate: Option<Text>,
    /// Source chromosome label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chromosome: Option<Text>,
    /// Source hg19 coordinates, without allele construction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hg19: Option<CosmicCoordinates>,
    /// Source nucleotide text, without repair or normalization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mut_nt: Option<Text>,
    /// Source frequency; units and denominator are not inferred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mut_freq: Option<FiniteNumber>,
    /// Every source tumor-site label in native order.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub tumor_site: Box<[Text]>,
}
biodata::wire_enum! {
    /// Relationship between a carried historical fragment and its aggregator record.
    CosmicIdentity {
        /// The fragment's exact allele equivalence has not been established.
        Unverified => "unverified",
    }
}
/// Historical coordinates exactly as carried by the component.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CosmicCoordinates {
    /// Source start value, without coordinate conversion.
    pub start: u64,
    /// Source end value, without coordinate conversion.
    pub end: u64,
}

/// Terms attached to one independently licensed source component.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComponentTerms {
    /// Publisher use category.
    pub usage: crate::Use,
    /// Publisher terms name.
    pub name: Text,
    /// Publisher terms page.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub url: Text,
}

/// Unchanged native context values associated with a source field path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PredictorContext {
    /// Native source field path.
    pub field: Text,
    /// Every native value in source array order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub values: Box<[Text]>,
}
/// One source-named component upload timestamp.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComponentUpload {
    /// Source subcollection name.
    pub name: Text,
    /// Unchanged source timestamp.
    pub date: crate::SourceDateTime,
}

biodata::wire_enum! {
    /// Provenance of the subject's accession qualification, without allele normalization.
    SubjectQualification {
        /// The adapter applied the approved chromosome/build accession table to an unchanged source description.
        AdapterDerivedReference => "adapter-derived-reference",
    }
}

/// Terms attached to a named carried component on a consumer response.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ComponentLabel {
    /// Native component name.
    pub component: crate::Text,
    /// Canonical publisher terms.
    pub terms: ComponentTerms,
}
