use crate::{native_text::NativeText, raw::Field};
use biodata::{Text, WikiPathwaysId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(super) struct Catalog {
    #[serde(rename = "pathwayInfo")]
    pub rows: Vec<Field<WireNative>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct WireNative {
    pub id: WikiPathwaysId,
    pub url: Text,
    pub species: Box<str>,
    pub name: Option<Field<NativeText>>,
    pub revision: Option<Field<NativeText>>,
    pub authors: Option<Field<Box<str>>>,
    pub description: Option<Field<Box<str>>>,
    #[serde(rename = "citedIn")]
    pub cited_in: Option<Field<Box<str>>>,
    pub datanodes: Option<Field<Box<str>>>,
    pub annotations: Option<Field<Box<str>>>,
}
/// Stored catalog metadata keeps previously serialized required values readable as Some.
#[derive(Debug, Deserialize, Serialize)]
pub(super) struct Native {
    pub id: WikiPathwaysId,
    pub url: Text,
    pub species: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Box<str>>,
    #[serde(rename = "citedIn", skip_serializing_if = "Option::is_none")]
    pub cited_in: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datanodes: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Box<str>>,
}
