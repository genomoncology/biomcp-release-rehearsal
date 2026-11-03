use crate::{Source, SourceRecord, Text, Variant};
use serde::{Deserialize, Serialize};
/// A compact identity card, including every provider candidate and explicit rejection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantCard {
    /// Identity outcome, without selecting the first candidate.
    pub resolution: VariantResolution,
    /// Responsible identity source.
    pub source: Source,
    /// All retained source identities.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub candidates: Box<[VariantHeader]>,
    /// Named input rejection from provider validation or a source-stated transcript gene mismatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection: Option<Text>,
}
impl super::ResponseBody for VariantCard {
    const SCHEMA: &'static str = "biomcp.variant-card/15";
    type Query = super::VariantGetQuery;
}

/// Consumer identity header with an exact source selection and compact annotation entity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantHeader {
    /// Source-qualified variant entity.
    pub identity: Variant,
    /// Checked caller ID used by a source-only section, without asserting resolved identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested: Option<crate::VariantId>,
    /// Source-paired transcript identifiers and unchanged coding and protein notation.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub transcripts: Box<[VariantTranscript]>,
    /// Source responsible for this identity.
    pub source: Source,
    /// Exact source record, including source keys that are expressions rather than identifiers.
    pub record: SourceRecord,
    /// Compact MyVariant annotations, attributed by their field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub myvariant: Option<Variant>,
    /// Explicitly requested source sections for this exact candidate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<Box<VariantSections>>,
    /// Terms for carried component identifiers on this consumer header.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub component_terms: Box<[crate::ComponentLabel]>,
}
biodata::wire_enum! {
    /// Identity resolution preserves candidates and source validation outcomes.
    VariantResolution {
        /// Checked source selector whose allele identity was not requested.
        Unresolved=>"unresolved",
        /// One checked provider identity.
        Resolved => "resolved",
        /// Several provider identities; none was selected.
        Candidates => "candidates",
        /// Provider reported no match.
        Absent => "absent",
        /// Input failed provider validation or disagreed with a source-stated transcript gene.
        Rejected => "rejected",
        /// Provider could not answer.
        Unavailable => "unavailable",
    }
}

/// Source-specific sections, absent unless explicitly selected.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantSections {
    /// Native prediction scores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predict: Option<crate::Section<crate::VariantPredictors>>,
    /// Native conservation scores.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conservation: Option<crate::Section<crate::VariantConservation>>,
    /// Historical MyVariant-associated COSMIC fragments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cosmic: Option<crate::Section<crate::VariantCosmic>>,

    /// ClinVar native assertion rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clinvar: Option<crate::Section<crate::VariantAssertions>>,
    /// ClinGen native assertion rows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub erepo: Option<crate::Section<crate::VariantAssertions>>,
    /// Explicit population source result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub population: Option<crate::Section<crate::VariantPopulation>>,
    /// Explicit GWAS Catalog source result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gwas: Option<crate::Section<crate::VariantGwas>>,
    /// Explicit keyed AlphaGenome numeric scorer result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alphagenome: Option<crate::Section<crate::VariantAlphaGenome>>,
    /// Explicit keyed OncoKB source result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oncokb: Option<crate::Section<crate::VariantOncokb>>,
    /// Explicit civic source result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub civic: Option<crate::Section<crate::VariantCivic>>,
    /// Explicit hotspots source result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotspots: Option<crate::Section<crate::VariantHotspots>>,
}

/// One native variant candidate page; its total and offset are in the shared envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantPage {
    /// Independent publisher terms for the declared dbNSFP gene/protein selector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector_terms: Option<crate::ComponentTerms>,
    /// Independent publisher terms for named ClinVar filters, present only when requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_terms: Option<crate::ComponentTerms>,

    /// Exact query sent to the named source, without rewriting raw query text.
    pub source_query: String,
    /// Assembly sent to the source, never inferred from a candidate label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<crate::Build>,
    /// Fallback candidates excluded because their reference allele was not confirmed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excluded: Option<VariantSearchExclusions>,
    /// Native source order, preserving every candidate including repeated source identities.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[VariantHeader]>,
}
/// Count of candidates withheld from a fallback search, with a stable reason.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantSearchExclusions {
    /// Native candidate count, independent of pagination's consumed positions.
    pub count: u32,
    /// Why these candidates could not be admitted.
    pub reason: VariantSearchExclusionReason,
}
biodata::wire_enum! {
    /// Fixed fallback exclusion reasons.
    VariantSearchExclusionReason {
        /// No allowed same-record annotation or agreeing reference sequence was available.
        ReferenceUnconfirmed => "reference-unconfirmed",
    }
}

impl super::ResponseBody for VariantPage {
    const SCHEMA: &'static str = "biomcp.variant-page/16";
    type Query = crate::VariantSearch;
}

/// A source-paired transcript annotation, without inferred transcript equivalence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantTranscript {
    /// Provider carrying this annotation.
    pub source: Source,
    /// Native component and field carrying the paired annotation.
    pub component: Text,
    /// Transcript identifier exactly as published, including its source version.
    pub transcript: Text,
    /// Source-stated gene symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene: Option<Text>,
    /// Source transcript notation from the coding field, including native n. or r. forms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_c: Option<Text>,
    /// Source protein notation, never translated from coding notation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hgvs_p: Option<Text>,
    /// Whether this row's source notation matched the declared caller notation.
    pub matched: bool,
}
