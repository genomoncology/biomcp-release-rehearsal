//! FDA-specific native details remain separate from the shared authorization event.
use biodata::{AuthorizationEvent, PartialDate, Text};
use serde::{Deserialize, Serialize};
/// One explicit native product within the selected FDA application.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugFdaProduct {
    /// Exact source product number, scoped by the response application.
    pub product_number: Text,
    /// Source brand wording, without inferred ingredient equivalence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<Text>,
    /// Complete source-listed ingredient and strength pairs.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub active_ingredients: Box<[DrugFdaIngredient]>,
    /// Native dosage form.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dosage_form: Option<Text>,
    /// Native route; absent and null remain unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<Text>,
    /// Product marketing status, independent of submission history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marketing_status: Option<Text>,
    /// Original source reference-drug designation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_drug: Option<Text>,
    /// Original source reference-standard designation when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_standard: Option<Text>,
    /// Native therapeutic-equivalence code when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub te_code: Option<Text>,
}
/// An unchanged native ingredient and strength, without local normalization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugFdaIngredient {
    /// Native ingredient name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Full source strength wording.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pub strength: Option<Text>,
}
/// One application-owned submission with common event facts and native FDA context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugFdaSubmission {
    /// Historical native status, type and status date; no product applicability is implied.
    pub event: AuthorizationEvent,
    /// Original submission number, scoped by application and event kind.
    pub submission_number: Text,
    /// Original review priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_priority: Option<Text>,
    /// Native submission class code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submission_class_code: Option<Text>,
    /// Native submission class description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub submission_class_code_description: Option<Text>,
    /// Native property collection; missing stays missing and an empty list stays empty.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub submission_property_type: Option<Box<[DrugFdaProperty]>>,
    /// Native documents; document dates remain separate from the submission status date.
    #[serde(default, skip_serializing_if = "biodata::absent_or_empty")]
    pub application_docs: Option<Box<[DrugFdaDocument]>>,
    /// Literal supplement purpose from its linked FDA letter; never a current-indication inference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplement_purpose: Option<DrugFdaSupplementPurpose>,
}
/// FDA-letter evidence for an efficacy supplement, independent of the current product label.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum DrugFdaSupplementPurpose {
    /// A complete source-stated purpose; may include several indications or other efficacy changes.
    Stated {
        /// FDA wording with PDF line wrapping normalized to spaces.
        text: Text,
        /// The document belongs to this submission in Drugs@FDA.
        document_id: Text,
        /// Published FDA letter URL.
        #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
        url: Text,
    },
    /// Missing evidence does not mean the supplement has no indication.
    Unavailable {
        /// What evidence is absent or could not be read.
        message: Box<str>,
        /// Selected native letter, when one was linked.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        document_id: Option<Text>,
        /// Selected native letter URL, when supplied.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<Text>,
        /// Actual download failure, when a request failed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<crate::Reason>,
    },
}
/// A source-stated submission property, without interpretation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugFdaProperty {
    /// Original property code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<Text>,
}
/// A native application document link, with embedded text only when its document ID is selected.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugFdaDocument {
    /// Source document identifier.
    pub id: Text,
    /// Original source URL.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<Text>,
    /// Native document date, independent of event dates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<PartialDate>,
    /// Bounded embedded letter text, present only when selected by document ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<crate::PdfText>,
    /// Original document type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Text>,
}
/// One candidate application and its independent source-listed products.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugFdaCandidate {
    /// Exact application identifier, shared with the retained application identity.
    pub id: biodata::DrugId,
    /// Exact brand wording shared by every product; omitted for zero or differing brands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pub name: Option<Text>,
    /// Exact native application identity; no product has been selected.
    pub identity: super::DrugHeader,
    /// Complete native product candidates within this application.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub products: Box<[DrugFdaProduct]>,
}
/// A source-owned application page preserving complete product context and source order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugFdaPage {
    /// Native applications and products; a phrase match never establishes equivalence.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[DrugFdaCandidate]>,
    /// Native application total, independent of product count.
    pub total: u64,
    /// Native dataset last_updated date, independent of event dates and retrieval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_updated: Option<crate::SourceUpdated>,
}
impl crate::ResponseBody for DrugFdaPage {
    const SCHEMA: &'static str = "biomcp.drug-fda-page/7";
    type Query = super::DrugSearch;
}
