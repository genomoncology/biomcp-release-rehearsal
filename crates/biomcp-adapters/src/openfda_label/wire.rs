//! Source label fields admit nonidentity values independently.
use crate::{native_text::NativeText, raw::Field};
use biodata::{DrugId, Text};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Record {
    /// Native revision identifier, distinct from its SPL set.
    pub id: DrugId,
    /// Native SPL set identifier.
    pub set_id: DrugId,
    /// Source-stated revision number.
    pub version: Option<Field<Text>>,
    /// Native YYYYMMDD revision date.
    pub effective_time: Option<Field<String>>,
    pub(super) openfda: Option<Field<Names>>,
    #[serde(flatten)]
    pub(super) sections: std::collections::BTreeMap<String, Field<Vec<Field<String>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Names {
    pub brand_name: Option<Field<Vec<Field<NativeText>>>>,
    pub generic_name: Option<Field<Vec<Field<NativeText>>>>,
    pub spl_set_id: Option<Field<Vec<Field<DrugId>>>>,
    pub application_number: Option<Field<Vec<Field<NativeText>>>>,
}
