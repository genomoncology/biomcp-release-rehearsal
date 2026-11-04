//! Preserve request molecule bindings even when a relation identity is malformed.
use crate::raw::Field;
use biomcp_fetch::{RawValue, decode_bundle_document};
use serde::{Deserialize, de::DeserializeOwned};
#[derive(Deserialize)]
pub(super) struct Molecule {
    pub molecule_chembl_id: String,
}
#[derive(Debug)]
pub(super) struct Row<T, Scope = Molecule> {
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
