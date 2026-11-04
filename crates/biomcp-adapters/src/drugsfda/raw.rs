//! Native FDA application scope; submission rows never acquire a product number.
use crate::openfda::{Envelope, Item};
pub(super) type Response = Envelope<Application>;
use crate::native_text::NativeText;
use biodata::{DrugId, PartialDate, Text};
use biomcp_fetch::BadReply;
use serde::Deserialize;
/// One exact native application with independent product and submission collections.
#[derive(Debug, Deserialize)]
#[serde(from = "super::wire::Application")]
pub(super) struct Application {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Rejected native source fields.
    pub problems: Vec<biomcp_types::Problem>,
    /// Native application identifier.
    pub application_number: DrugId,
    /// Original sponsor wording.
    pub sponsor_name: Option<biodata::Text>,
    /// Native products in publisher order.
    pub products: Vec<Product>,
    /// Application-owned submission history in publisher order.
    pub submissions: Vec<Submission>,
    /// Optional source-supplied cross references.
    pub openfda: Option<OpenFda>,
}
/// Source-given label links, without an inferred product association.
#[derive(Debug)]
pub(super) struct OpenFda {
    /// Complete explicit SPL set identifiers.
    pub spl_set_id: Option<Vec<DrugId>>,
}
/// One native product, independent of application history.
#[derive(Debug)]
pub(super) struct Product {
    /// Exact native product selector.
    pub product_number: Text,
    /// Native product name.
    pub brand_name: Option<biodata::Text>,
    /// Source-listed ingredients and strengths.
    pub active_ingredients: Vec<Ingredient>,
    /// Original formulation wording.
    pub dosage_form: Option<Text>,
    /// Native route, which may be null.
    pub route: Option<Text>,
    /// Original product marketing status, never a computed approval flag.
    pub marketing_status: Option<Text>,
    /// Native reference-drug wording.
    pub reference_drug: Option<Text>,
    /// Native reference-standard wording, omitted in some records.
    pub reference_standard: Option<Text>,
    /// Native therapeutic-equivalence code when given.
    pub te_code: Option<Text>,
}
/// Source-listed ingredient and unchanged strength text.
#[derive(Debug)]
pub(super) struct Ingredient {
    /// Native ingredient name, without normalization.
    pub name: Option<biodata::Text>,
    /// Full native strength wording.
    pub strength: Option<NativeText>,
}
/// An application submission; the publisher supplies no product applicability.
#[derive(Debug)]
pub(super) struct Submission {
    /// Native submission category.
    pub submission_type: Option<Text>,
    /// Native submission identifier within the application and type.
    pub submission_number: Text,
    /// Native historical status.
    pub submission_status: Option<Text>,
    /// Native YYYYMMDD status date, validated before cache admission.
    pub submission_status_date: Option<PartialDate>,
    /// Original review priority.
    pub review_priority: Option<Text>,
    /// Original submission class code.
    pub submission_class_code: Option<Text>,
    /// Original class description.
    pub submission_class_code_description: Option<NativeText>,
    /// Native property collection.
    pub submission_property_type: Option<Vec<Property>>,
    /// Native documents with their independent date roles.
    pub application_docs: Option<Vec<Document>>,
}
/// One native submission property.
#[derive(Debug)]
pub(super) struct Property {
    /// Original property code.
    pub code: Option<Text>,
}
/// A native application document, never fetched by the adapter.
#[derive(Debug)]
pub(super) struct Document {
    /// Source document identifier.
    pub id: Text,
    /// Original document URL.
    pub url: Option<Text>,
    /// Native YYYYMMDD document date.
    pub date: Option<PartialDate>,
    /// Native document kind.
    pub r#type: Option<Text>,
}

/// Convert the source date without assigning it an approval meaning.
pub(crate) fn date(value: &str) -> Result<PartialDate, BadReply> {
    if value.len() != 8 || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(BadReply);
    }
    format!("{}-{}-{}", &value[..4], &value[4..6], &value[6..])
        .parse()
        .map_err(|_| BadReply)
}
impl Application {
    fn check(&self) -> Result<(), BadReply> {
        if !biomcp_types::DrugSource::Drugsfda.accepts(&self.application_number)
            || self.products.iter().enumerate().any(|(i, p)| {
                self.products
                    .iter()
                    .take(i)
                    .any(|old| old.product_number == p.product_number)
            })
            || self
                .openfda
                .as_ref()
                .and_then(|o| o.spl_set_id.as_ref())
                .is_some_and(|ids| {
                    ids.iter()
                        .any(|id| !biomcp_types::DrugSource::OpenfdaLabel.accepts(id))
                })
        {
            return Err(BadReply);
        }
        Ok(())
    }
}

impl Item for Application {
    fn check(&self) -> Result<(), BadReply> {
        Application::check(self)
    }
}
