use super::{DrugGet, DrugHeader};
use crate::{CivicClaimPage, CivicMember, CivicStatement, SourceRecord};
use biodata::Text;
use serde::{Deserialize, Serialize};

/// Independent native therapy claim collections, without profile count inference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugCivicStreams {
    /// Evidence items filtered by the selected native therapy and optional profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<DrugCivicPage>,
    /// Assertions filtered independently by the selected native therapy and optional profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assertions: Option<DrugCivicPage>,
}
/// One native claim stream and its own continuation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugCivicPage {
    /// Native claim order; each row carries the profile to which that claim belongs.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub records: Box<[DrugCivicClaim]>,
    /// Stream totals and continuation, independent of every other stream.
    pub page: CivicClaimPage<DrugGet>,
}
/// One native claim with its complete source therapy set and molecular profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugCivicClaim {
    /// Source molecular profile context; its member expression is never collapsed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<DrugCivicProfile>,
    /// Existing source claim mapping, including unmapped native facts.
    pub claim: CivicStatement,
    /// Complete native therapy identities, without cross-source equivalence.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub therapies: Box<[DrugHeader]>,
    /// Native combination interaction label when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub therapy_interaction_type: Option<Text>,
    /// Missing source context prevents a keyed variant annotation next command.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub oncokb_missing: Box<[DrugOncokbMissing]>,
    /// Scoped lookup of a source-listed genomic description, with no evidence transfer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oncokb_lookup: Option<DrugOncokbLookup>,
}
/// Profile context embedded in a therapy claim, with no invented profile totals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugCivicProfile {
    /// Native profile identity and link.
    pub record: SourceRecord,
    /// Complete source expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
    /// Exact source expression with native variant references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_name: Option<Text>,
    /// Checked source profile subject, independent of an allele join or a clinical claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<crate::VariantRef>,
    /// Source complex-profile flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compound: Option<bool>,
    /// Every source member, including members without a common-model subject.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub members: Box<[DrugCivicMember]>,
}

/// A drug claim member with its source-owned build, without altering variant responses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugCivicMember {
    /// Existing native member facts and unchanged HGVS descriptions.
    #[serde(flatten)]
    pub member: CivicMember,
    /// Native coordinates.referenceBuild spelling, including unsupported builds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_build: Option<Text>,
}

biodata::wire_enum! {
    /// Context absent or insufficient for an exact source-owned annotation command.
    DrugOncokbMissing {
        /// No single checked gene in the native profile.
        Gene => "gene",
        /// No explicit supported native reference build.
        Build => "build",
        /// No single genomic description on that build, or a compound profile.
        Variant => "variant",
        /// No source claim tumor label.
        TumorType => "tumor-type",
    }
}

/// Link one source member to a checked variant lookup in its claim's tumor context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugOncokbLookup {
    /// Native CIViC member ID to which the genomic description belongs.
    pub member: Text,
    /// Scope visible alongside the generated command.
    pub label: Text,
    /// Checked owning-entity request; input preserves the selected source HGVS.
    pub query: crate::VariantGet,
}
