//! Check requested selectors even when association identity cannot be admitted.
use super::raw::{Association, EffectAllele, Trait};
use crate::raw::Field;
use biodata::Text;
use biomcp_fetch::{RawValue, decode_bundle_document};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Selectors {
    pub accession_id: Option<Field<Text>>,
    pub pubmed_id: Option<Field<String>>,
    pub snp_allele: Option<Field<Vec<Field<EffectAllele>>>>,
    pub efo_traits: Option<Field<Vec<Field<Trait>>>>,
}
impl Selectors {
    pub(super) fn alleles(&self) -> impl Iterator<Item = &EffectAllele> {
        self.snp_allele
            .as_ref()
            .and_then(Field::as_ref)
            .into_iter()
            .flatten()
            .filter_map(Field::as_ref)
    }
    pub(super) fn traits(&self) -> impl Iterator<Item = &Trait> {
        self.efo_traits
            .as_ref()
            .and_then(Field::as_ref)
            .into_iter()
            .flatten()
            .filter_map(Field::as_ref)
    }
}
#[derive(Debug)]
pub(super) struct Row {
    pub selectors: Selectors,
    pub value: Field<Association>,
}
impl<'de> Deserialize<'de> for Row {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(deserializer)?;
        let selectors =
            decode_bundle_document(raw.get().as_bytes()).map_err(serde::de::Error::custom)?;
        let value =
            decode_bundle_document(raw.get().as_bytes()).map_err(serde::de::Error::custom)?;
        Ok(Self { selectors, value })
    }
}
