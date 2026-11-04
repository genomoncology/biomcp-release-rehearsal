//! WHO prequalification rows preserve source cells without authorization inference.
use crate::{DrugSource, QueryInvalid, ResponseBody};
use biodata::Text;
use serde::{Deserialize, Serialize};
/// Every published cell of one WHO finished-product record, including empty cells.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugWhoRow {
    /// Exact native WHO Reference Number, including spaces and punctuation.
    pub reference: Text,
    /// Native combined INN, Dosage Form and Strength cell; never split or interpreted.
    pub inn_dosage_form_strength: Box<str>,
    /// Native Product Type, retained even when empty.
    pub product_type: Box<str>,
    /// Native Therapeutic Area wording.
    pub therapeutic_area: Box<str>,
    /// Native Applicant wording.
    pub applicant: Box<str>,
    /// Native Dosage Form cell, independent of the combined cell.
    pub dosage_form: Box<str>,
    /// Native Basis of Listing wording.
    pub basis_of_listing: Box<str>,
    /// Native Date of Prequalification text, including spacing and empty strings.
    pub date_of_prequalification: Box<str>,
}
/// Source-ordered WHO matches; prequalification does not establish US/EU authorization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugWhoPage {
    /// Bounded native rows in export order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[DrugWhoRow]>,
    /// Matching rows in the complete local snapshot.
    pub total: u64,
}
/// Source-scoped native rows for one exact WHO reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct DrugWhoCard {
    /// Exact requested and source-matched reference.
    pub reference: Text,
    /// Every native row for that reference, without invented fields.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub native_rows: Box<[DrugWhoRow]>,
}
/// WHO exact-reference selector keeps the shared DrugId contract unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct DrugWhoGet(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// Exact WHO reference number, including native spaces; no name or alias resolution.
    #[cfg_attr(feature = "schema", schemars(extend("x-positional" = 0)))]
    input: Text,
    /// Explicit source who-pq; prequalification is source-scoped.
    source: DrugSource,
}
crate::query::validation::checked_query!(DrugWhoGet, Fields);
impl TryFrom<Fields> for DrugWhoGet {
    type Error = QueryInvalid;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        if fields.source != DrugSource::WhoPq {
            return Err(QueryInvalid::new(&["source"]));
        }
        Ok(Self(fields))
    }
}
impl DrugWhoGet {
    /// Exact requested native reference.
    pub fn input(&self) -> &Text {
        &self.0.input
    }
    /// Construct a checked action for a native WHO reference.
    pub fn new(input: Text) -> Self {
        Self(Fields {
            input,
            source: DrugSource::WhoPq,
        })
    }
}
impl ResponseBody for DrugWhoPage {
    const SCHEMA: &'static str = "biomcp.drug-who-page/5";
    type Query = crate::DrugSearch;
}
impl ResponseBody for DrugWhoCard {
    const SCHEMA: &'static str = "biomcp.drug-who-card/3";
    type Query = DrugWhoGet;
}
