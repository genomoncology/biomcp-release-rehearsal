use crate::{PageInfo, SourceRecord, Statement, Text, VariantGet, VariantRef};
use serde::{Deserialize, Serialize};

/// CIViC molecular profiles with their original membership and separate source claims.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantCivic {
    /// Source profiles in native order; name candidates carry no attached statements.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub profiles: Box<[CivicProfile]>,
    /// Disclosure bounds for this section.
    pub page: PageInfo,
    /// Continue this section without changing the input or source selectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<VariantGet>,
}

/// One source molecular profile, preserving compound expressions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "CivicProfile_{Continuation}"))]
pub struct CivicProfile<Continuation = VariantGet> {
    /// Native profile identity and link.
    pub record: SourceRecord,
    /// Whole source profile label, including its logical expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
    /// Exact source expression with native variant references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_name: Option<Text>,
    /// Checked source profile subject, independent of an allele join or a clinical claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<crate::VariantRef>,
    /// Native complex-profile flag; a member match does not satisfy the whole profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compound: Option<bool>,
    /// Whether a source member was joined to the requested exact identity.
    pub membership: CivicMembership,
    /// Every source member reference, without collapsing compound profiles.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub members: Box<[CivicMember]>,
    /// Native evidence count, independent of the displayed page.
    pub evidence_count: u64,
    /// Native assertion count, independent of the displayed page.
    pub assertion_count: u64,
    /// Source claims; a compound claim retains a profile subject.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub statements: Box<[CivicStatement]>,
    /// Native evidence stream and its independent continuation.
    pub evidence_page: CivicClaimPage<Continuation>,
    /// Native assertion stream and its independent continuation.
    pub assertion_page: CivicClaimPage<Continuation>,
}

/// A native CIViC member, including members without a common-model subject.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivicMember {
    /// Native variant identity and link.
    pub record: SourceRecord,
    /// Whole source variant label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
    /// Native feature label, including fusion features.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature: Option<Text>,
    /// Native feature identifier, distinct from the variant identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_record: Option<crate::Curie>,
    /// Every native notation, without selecting a transcript or repairing a spelling.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub hgvs_descriptions: Box<[Text]>,
    /// Source-qualified subject when supported by the common model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<VariantRef>,
}

/// Native stream bounds; evidence and assertions have separate cursors.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "CivicClaimPage_{Continuation}"))]
pub struct CivicClaimPage<Continuation = VariantGet> {
    /// Number of native rows displayed in this stream.
    pub returned: u32,
    /// Native total for this stream.
    pub total: u64,
    /// Continue the same source candidate, profile and claim stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<Continuation>,
}

/// One CIViC claim with native labels that exceed the common statement model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivicStatement {
    /// Native claim cannot be represented by the common subject or proposition model.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub statement_unmapped: bool,
    /// Common source-attributed claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement: Option<Statement>,
    /// Complete native claim facts when the common subject or proposition is unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<CivicNativeClaim>,
    /// Native claim identity, retained even when no common proposition applies.
    pub record: SourceRecord,
    /// Source claim category, without a fallback proposition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<super::CivicKind>,
    /// Native curation state, without implying that every row is accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<super::CivicStatus>,
    /// Original provider direction; disputes never implies the opposite effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Text>,
    /// Original significance, including when the classification holds an AMP tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub significance: Option<super::CivicSignificance>,
    /// Exact nullable native AMP value, including Not Applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amp_level: Option<Text>,
    /// Native relationship between therapies; the common statement cannot express it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub therapy_interaction_type: Option<Text>,
}

/// Source facts retained without inventing a common-model subject or proposition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivicNativeClaim {
    /// Native classification when its source text is nonempty; significance remains on the claim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<crate::Term>,
    /// Native level, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strength: Option<crate::Term>,
    /// Source disease context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<CivicCondition>,
    /// Native therapies in source order.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub therapies: Box<[CivicTherapy]>,
    /// Native origin, when stated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allele_origin: Option<crate::Term>,
    /// Native publication identities.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub citations: Box<[crate::Pmid]>,
    /// Original source provenance.
    pub attribution: crate::Attribution,
}

/// Native disease fields remain independent when one value is malformed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivicCondition {
    /// Source DOID, without a disease-name conversion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<biodata::Curie>,
    /// Complete source disease name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
}
/// One native therapy member; a label does not imply a source identifier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivicTherapy {
    /// Native therapy identity when requested by this source route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<biodata::Identifier<u64>>,
    /// Complete source therapy name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
}

biodata::wire_enum! {
    /// Identity strength of a CIViC profile lookup.
    CivicMembership {
        /// Source member identity was checked against the requested variant.
        Exact => "exact",
        /// Only a source name matched; no claim is attached to an allele.
        Candidate => "candidate",
    }
}
