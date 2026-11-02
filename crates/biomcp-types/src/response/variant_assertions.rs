use crate::{
    Attribution, Concept, Curie, PageInfo, PartialDate, Pmid, Statement, Text, VariantAssertionsGet,
};
use serde::{Deserialize, Serialize};
/// One source-ordered page, with native record and classification boundaries intact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantAssertions {
    /// Source assertion rows in native order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub assertions: Box<[VariantAssertion]>,
    /// Page bounds over native rows.
    pub page: PageInfo,
    /// Typed continuation preserving source and selectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<VariantAssertionsGet>,
}
impl super::ResponseBody for VariantAssertions {
    const SCHEMA: &'static str = "biomcp.variant-assertions/8";
    type Query = VariantAssertionsGet;
}
/// A native record/domain pair, never a vote or merged classification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct VariantAssertion {
    /// Aggregate/submission/document context.
    pub kind: AssertionKind,
    /// Source classification domain.
    pub domain: AssertionDomain,
    /// The native conclusion has no common-model statement.
    #[serde(default, skip_serializing_if = "is_false")]
    pub statement_unmapped: bool,
    /// Supported mapping into the common model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement: Option<Statement>,
    /// Native assertion whose category the common model cannot represent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unmapped: Option<UnmappedAssertion>,
    /// Original source detail, returned only for an explicitly selected record.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<AssertionDetail>,
}
impl VariantAssertion {
    /// Record provenance without relying on the clinical mapping.
    pub fn attribution(&self) -> Option<&Attribution> {
        self.statement
            .as_ref()
            .map(|s| &s.attribution)
            .or_else(|| self.unmapped.as_ref().map(|s| &s.attribution))
    }
}
/// Source conclusion and provenance retained without a shared statement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct UnmappedAssertion {
    /// Unchanged source label, when valid and supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<Text>,
    /// Original record provenance.
    pub attribution: Attribution,
    /// Native review status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_status: Option<Text>,
    /// Native conditions, including identifiers without supplied labels.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub conditions: Box<[AssertionCondition]>,
}
/// Source details that do not fit the compact shared statement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AssertionDetail {
    /// Native somatic impact assertion type, without deriving a predicate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact_assertion_type: Option<Text>,
    /// Native somatic significance, independent of the tier label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact_clinical_significance: Option<Text>,
    /// Native therapy text, without splitting or assigning it to a predicate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact_therapies: Option<Text>,
    /// Native classification comments, without extracting clinical claims.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub comments: Box<[Text]>,
    /// Assertion method labels, not inferred from clinical testing.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub methods: Box<[Text]>,
    /// All source conditions; ID-only conditions stay ID-only.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub conditions: Box<[AssertionCondition]>,
    /// Native trait relationships, kept separate from assertion conditions.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub condition_relationships: Box<[AssertionConditionRelationship]>,
    /// Source origin labels without reducing multiple observations.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub origins: Box<[Text]>,
    /// Source citations, including assertion-method citations.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub citations: Box<[Pmid]>,
    /// Source prose, kept separately from structured criteria.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Text>,
    /// Criteria explicitly marked unmet.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub unmet_criteria: Box<[Text]>,
    /// Native met criteria retained when no common statement can carry them.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub met_criteria: Box<[Text]>,
    /// Native gene context retained when the conclusion cannot map to a statement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene_context: Option<biodata::SourceGeneRef>,
    /// Source publication date; never an inferred evaluation date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<PartialDate>,
    /// Source approval date, distinct from evaluation and publication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<PartialDate>,
    /// Source document facts for an explicitly selected historical version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<AssertionDocument>,
    /// Native document versions offered by the source.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub versions: Box<[Text]>,
}
/// A source trait relationship without assigning a clinical interpretation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AssertionConditionRelationship {
    /// Relationship type exactly supplied by the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_type: Option<Text>,
    /// Conditions on the source trait that owns this relationship.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub source_conditions: Box<[AssertionCondition]>,
    /// Conditions named by the related source trait.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub related_conditions: Box<[AssertionCondition]>,
}
/// A native condition with independently optional identifier and label.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AssertionCondition {
    /// Native database name without inventing a CURIE prefix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<Text>,
    /// Native accession text, preserved even when no CURIE namespace is recognized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accession: Option<Text>,
    /// Identifier exactly supplied by the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Curie>,
    /// Label only when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<Text>,
}
impl AssertionCondition {
    /// A common concept exists only when the source supplied a label.
    pub fn concept(&self) -> Option<Concept> {
        self.label.clone().map(|label| Concept {
            id: self.id.clone(),
            label,
        })
    }
}
biodata::wire_enum! {
    /// Native record scope.
    AssertionKind {
        /// Variant aggregate.
        Vcv=>"vcv",
        /// Variant-condition aggregate.
        Rcv=>"rcv",
        /// Submitted classification.
        Scv=>"scv",
        /// ClinGen expert-panel document.
        Erepo=>"erepo",
    }
}
biodata::wire_enum! {
    /// Source classification domains remain separate.
    AssertionDomain {
        /// Germline source classification.
        Germline=>"germline",
        /// Somatic clinical impact.
        SomaticClinicalImpact=>"somatic_clinical_impact",
        /// Somatic oncogenicity.
        Oncogenicity=>"oncogenicity",
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// Native SEPIO facts kept separate from common interpretation fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AssertionDocument {
    /// Source UUID checked against the selected summary link.
    pub uuid: Text,
    /// Native interpretation identifier, separate from the summary PCER identifier.
    pub native_id: Text,
    /// Native interpretation type.
    pub document_type: Text,
    /// Native outcome identifier without assigning a common coding system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_id: Option<Text>,
    /// Native outcome IRI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_iri: Option<Text>,
    /// Assessments directly supporting the interpretation.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub assessments: Box<[AssertionAssessment]>,
}
/// Source assessment labels and identifiers remain independent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AssertionAssessment {
    /// Native criterion code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion: Option<Text>,
    /// Native criterion identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion_id: Option<Text>,
    /// Native criterion IRI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criterion_iri: Option<Text>,
    /// Native default strength, distinct from an applied evidence strength.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_strength: Option<Text>,
    /// Native default-strength identifier; not an applied evidence strength.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_strength_id: Option<Text>,
    /// Native default-strength IRI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_strength_iri: Option<Text>,
    /// Source outcome label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Text>,
    /// The supplied outcome identifier contradicts its supplied Met/Not Met label.
    #[serde(default, skip_serializing_if = "is_false")]
    pub outcome_conflict: bool,
    /// Source outcome identifier, including known conflicting ID/label pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_id: Option<Text>,
    /// Source outcome IRI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_iri: Option<Text>,
}
