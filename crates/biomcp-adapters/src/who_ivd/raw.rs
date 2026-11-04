use biodata::{WhoIvdRow, WhoProductId};
use biomcp_fetch::{KeyKind, Keys, Row};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Native {
    #[serde(rename = "WHO Product ID")]
    pub id: WhoProductId,
    #[serde(rename = "Product name")]
    pub product_name: Box<str>,
    #[serde(rename = "Product Code")]
    product_code: Box<str>,
    #[serde(rename = "Assay Format")]
    assay_format: Box<str>,
    #[serde(rename = "Manufacturer name")]
    pub manufacturer: Box<str>,
    #[serde(rename = "Pathogen/Disease/Marker")]
    pub marker: Box<str>,
    #[serde(rename = "Regulatory Version")]
    regulatory_version: Box<str>,
    #[serde(rename = "Year prequalification")]
    prequalification_year: Box<str>,
}
impl Row for Native {
    fn keys(&self) -> Keys {
        Keys {
            exact: vec![(KeyKind::Id, self.id.to_string().into_boxed_str())],
            words: Vec::new(),
        }
    }
}
impl From<Native> for WhoIvdRow {
    fn from(row: Native) -> Self {
        Self {
            product_name: row.product_name,
            product_code: row.product_code,
            assay_format: row.assay_format,
            manufacturer: row.manufacturer,
            marker: row.marker,
            regulatory_version: row.regulatory_version,
            prequalification_year: row.prequalification_year,
        }
    }
}
