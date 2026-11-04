use crate::{native_text::NativeText, raw::Field};
use biodata::{AminoAcids, ProteinIsoformStatus, ProteinStructureProperty, Text, UniProtIsoform};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Feature {
    #[serde(rename = "type")]
    pub kind: Text,
    pub location: Location,
    pub description: Option<Field<String>>,
    pub feature_id: Option<Field<NativeText>>,
    pub alternative_sequence: Option<Field<Alternative>>,
    pub evidences: Option<Field<Vec<Field<Evidence>>>>,
}
#[derive(Debug, Deserialize)]
pub(in crate::uniprot) struct Location {
    pub start: Boundary,
    pub end: Boundary,
    pub sequence: Option<UniProtIsoform>,
}
#[derive(Debug, Deserialize)]
pub(in crate::uniprot) struct Boundary {
    pub value: Option<u32>,
    pub modifier: Modifier,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub(in crate::uniprot) enum Modifier {
    Exact,
    Outside,
    Unknown,
    Unsure,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Alternative {
    pub original_sequence: Option<Field<AminoAcids>>,
    pub alternative_sequences: Option<Field<Vec<Field<AminoAcids>>>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Evidence {
    pub evidence_code: Text,
    pub source: Option<Field<NativeText>>,
    pub id: Option<Field<NativeText>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Comment {
    pub comment_type: String,
    pub molecule: Option<Field<NativeText>>,
    pub texts: Option<Field<Vec<Field<Paragraph>>>>,
    pub isoforms: Option<Field<Vec<Field<Isoform>>>>,
}
#[derive(Debug, Deserialize)]
pub(in crate::uniprot) struct Paragraph {
    #[serde(deserialize_with = "crate::native_text::deserialize")]
    pub value: Text,
    pub evidences: Option<Field<Vec<Field<Evidence>>>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::uniprot) struct Isoform {
    pub name: Option<Field<super::raw::Value>>,
    pub synonyms: Option<Field<Vec<Field<super::raw::Value>>>>,
    pub isoform_ids: Option<Field<Vec<Field<UniProtIsoform>>>>,
    pub sequence_ids: Option<Field<Vec<Field<NativeText>>>>,
    pub isoform_sequence_status: Option<Field<ProteinIsoformStatus>>,
}
#[derive(Debug, Deserialize)]
pub(in crate::uniprot) struct CrossReference {
    pub database: Text,
    pub id: Text,
    pub properties: Option<Field<Vec<Field<ProteinStructureProperty>>>>,
}
