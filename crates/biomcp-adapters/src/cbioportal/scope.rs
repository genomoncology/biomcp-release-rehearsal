//! Check native request scope independently from each row's entity identity.
use crate::raw::Field;
use biomcp_fetch::{RawValue, decode_bundle_document};
use serde::{Deserialize, de::DeserializeOwned};
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Study {
    pub study_id: biodata::StudyId,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Mutation {
    pub study_id: biodata::Text,
    pub molecular_profile_id: biodata::Text,
    pub entrez_gene_id: u32,
}
#[derive(Debug)]
pub(super) struct Row<T, Scope> {
    pub scope: Scope,
    pub value: Field<T>,
}
impl<'de, T: DeserializeOwned, Scope: DeserializeOwned> Deserialize<'de> for Row<T, Scope> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        let scope =
            decode_bundle_document(raw.get().as_bytes()).map_err(serde::de::Error::custom)?;
        let value =
            decode_bundle_document(raw.get().as_bytes()).map_err(serde::de::Error::custom)?;
        Ok(Self { scope, value })
    }
}
