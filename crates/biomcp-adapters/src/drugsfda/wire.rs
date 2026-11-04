//! Native FDA nonidentity fields admit independently.
use crate::{native_text::NativeText, raw::Field};
use biodata::{DrugId, Text};
use serde::Deserialize;
/// One exact native application with independent product and submission collections.
#[derive(Debug, Deserialize)]
pub(super) struct Application {
    /// Native application identifier.
    pub application_number: DrugId,
    /// Original sponsor wording.
    pub sponsor_name: Option<Field<NativeText>>,
    /// Native products in publisher order.
    pub products: Option<Field<Vec<Field<Product>>>>,
    /// Application-owned submission history in publisher order.
    pub submissions: Option<Field<Vec<Field<Submission>>>>,
    /// Optional source-supplied cross references.
    pub openfda: Option<Field<OpenFda>>,
}
/// Source-given label links, without an inferred product association.
#[derive(Debug, Deserialize)]
pub(super) struct OpenFda {
    /// Complete explicit SPL set identifiers.
    pub spl_set_id: Option<Field<Vec<Field<DrugId>>>>,
}
/// One native product, independent of application history.
#[derive(Debug, Deserialize)]
pub(super) struct Product {
    /// Exact native product selector.
    pub product_number: Text,
    /// Native product name.
    pub brand_name: Option<Field<NativeText>>,
    /// Source-listed ingredients and strengths.
    pub active_ingredients: Option<Field<Vec<Field<Ingredient>>>>,
    /// Original formulation wording.
    pub dosage_form: Option<Field<Text>>,
    /// Native route, which may be null.
    pub route: Option<Field<Text>>,
    /// Original product marketing status, never a computed approval flag.
    pub marketing_status: Option<Field<Text>>,
    /// Native reference-drug wording.
    pub reference_drug: Option<Field<Text>>,
    /// Native reference-standard wording, omitted in some records.
    pub reference_standard: Option<Field<Text>>,
    /// Native therapeutic-equivalence code when given.
    pub te_code: Option<Field<Text>>,
}
/// Source-listed ingredient and unchanged strength text.
#[derive(Debug, Deserialize)]
pub(super) struct Ingredient {
    /// Native ingredient name, without normalization.
    pub name: Option<Field<NativeText>>,
    /// Full native strength wording.
    pub strength: Option<Field<NativeText>>,
}
/// An application submission; the publisher supplies no product applicability.
#[derive(Debug, Deserialize)]
pub(super) struct Submission {
    /// Native submission category.
    pub submission_type: Option<Field<Text>>,
    /// Native submission identifier within the application and type.
    pub submission_number: Text,
    /// Native historical status.
    pub submission_status: Option<Field<Text>>,
    /// Native YYYYMMDD status date, validated before cache admission.
    pub submission_status_date: Option<Field<String>>,
    /// Original review priority.
    pub review_priority: Option<Field<Text>>,
    /// Original submission class code.
    pub submission_class_code: Option<Field<Text>>,
    /// Original class description.
    pub submission_class_code_description: Option<Field<NativeText>>,
    /// Native property collection.
    pub submission_property_type: Option<Field<Vec<Field<Property>>>>,
    /// Native documents with their independent date roles.
    pub application_docs: Option<Field<Vec<Field<Document>>>>,
}
/// One native submission property.
#[derive(Debug, Deserialize)]
pub(super) struct Property {
    /// Original property code.
    pub code: Option<Field<Text>>,
}
/// A native application document, never fetched by the adapter.
#[derive(Debug, Deserialize)]
pub(super) struct Document {
    /// Source document identifier.
    pub id: Text,
    /// Original document URL.
    pub url: Option<Field<Text>>,
    /// Native YYYYMMDD document date.
    pub date: Option<Field<String>>,
    /// Native document kind.
    pub r#type: Option<Field<Text>>,
}
