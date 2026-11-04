//! Typed native Open Targets responses; no dataset release is invented.
use crate::raw::Field;
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
/// GraphQL response with source errors kept distinct from empty data.
#[derive(Debug, Deserialize)]
pub(super) struct Response<T> {
    /// Source result data.
    pub data: T,
    errors: Option<Vec<serde::de::IgnoredAny>>,
}
impl<T: for<'de> Deserialize<'de>> Reply for Response<T> {
    fn check(&self) -> Result<(), BadReply> {
        if self.errors.as_ref().is_some_and(|e| !e.is_empty()) {
            Err(BadReply)
        } else {
            Ok(())
        }
    }
}
/// Source candidate collection.
#[derive(Debug, Deserialize)]
pub(super) struct SearchData {
    /// Native candidate page.
    pub search: Search,
}
/// Candidate rows retain their native order.
#[derive(Debug, Deserialize)]
pub(super) struct Search {
    /// Source total.
    pub total: u64,
    /// Source candidates.
    pub(super) hits: Vec<Field<Hit>>,
}
/// One native candidate identity.
#[derive(Debug, Deserialize)]
pub(super) struct Hit {
    /// Native ChEMBL identifier.
    pub id: String,
    /// Native source name.
    pub(super) name: Option<Field<String>>,
    /// Native record entity.
    pub(super) entity: String,
}
/// Exact-ID response.
#[derive(Debug, Deserialize)]
pub(super) struct DrugData {
    /// Missing source record remains absent.
    pub drug: Option<Drug>,
}
/// Native source drug and its complete returned collections.
#[derive(Debug, Deserialize)]
pub(super) struct Drug {
    /// Exact native ChEMBL identifier.
    pub id: String,
    /// Native source name.
    pub(super) name: Option<Field<String>>,
    /// Complete native indication collection without invented server paging.
    pub indications: Rows<Indication>,
    /// Complete native mechanism collection without invented server paging.
    #[serde(rename = "mechanismsOfAction")]
    pub mechanisms_of_action: Rows<Mechanism>,
}
/// A native returned collection.
#[derive(Debug, Deserialize)]
pub(super) struct Rows<T> {
    /// Native ordered rows.
    pub(super) rows: Vec<Field<T>>,
}
/// Source indication and clinical stage, without regulatory interpretation.
#[derive(Debug, Deserialize)]
pub(super) struct Indication {
    /// Native clinical stage spelling.
    #[serde(rename = "maxClinicalStage")]
    pub(super) max_clinical_stage: Option<Field<String>>,
    /// Native disease identifier and label.
    pub disease: Disease,
}
/// A native disease reference without a name join.
#[derive(Debug, Deserialize)]
pub(super) struct Disease {
    /// Native source identifier.
    pub id: String,
    /// Original source label.
    pub(super) name: Option<Field<String>>,
}
/// Native mechanism and its complete target members.
#[derive(Debug, Deserialize)]
pub(super) struct Mechanism {
    /// Complete original mechanism wording.
    #[serde(rename = "mechanismOfAction")]
    pub(super) mechanism_of_action: Option<Field<String>>,
    /// Original action type.
    #[serde(rename = "actionType")]
    pub(super) action_type: Option<Field<String>>,
    /// Complete native target member context.
    pub(super) targets: Option<Field<Vec<Field<Target>>>>,
}
/// Native target gene identities and labels.
#[derive(Debug, Deserialize)]
pub(super) struct Target {
    /// Native Ensembl gene ID.
    pub id: String,
    /// Original gene symbol.
    #[serde(rename = "approvedSymbol")]
    pub(super) approved_symbol: Option<Field<String>>,
    /// Original gene name.
    #[serde(rename = "approvedName")]
    pub(super) approved_name: Option<Field<String>>,
}
