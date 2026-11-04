use crate::native_text::NativeText;
use biodata::{CellLineXref, CellosaurusAccession, PartialDate, Text};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Response {
    Records {
        #[serde(rename = "Cellosaurus")]
        data: Data,
    },
    Missing {
        code: u16,
        message: String,
    },
}
#[derive(Debug, Deserialize)]
pub(super) struct Data {
    #[serde(rename = "cell-line-list")]
    pub rows: Vec<Record>,
}
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedRecord")]
pub(super) struct Record {
    pub problems: Vec<biomcp_types::Problem>,
    pub accession_list: Vec<crate::raw::Field<Accession>>,
    pub name_list: Vec<Name>,
    pub category: Option<Text>,
    pub species_list: Option<crate::strict_input::Input<Box<[CellLineXref]>>>,
    pub disease_list: Option<crate::strict_input::Input<Box<[CellLineXref]>>>,
    pub age: Option<Text>,
    pub sex: Option<biodata::CellLineSex>,
    pub created: Option<PartialDate>,
    pub last_updated: Option<PartialDate>,
    pub entry_version: Option<Text>,
    pub comment_list: Option<Box<[Comment]>>,
    pub xref_list: Option<Box<[Xref]>>,
    pub sequence_variation_list: Option<Box<[Variation]>>,
}
// Native xrefs can carry additional publisher fields, unlike the public projection.
#[derive(Debug, Deserialize)]
pub(super) struct Xref {
    pub database: Text,
    pub accession: Text,
    pub label: Option<crate::raw::Field<NativeText>>,
    pub url: Option<crate::raw::Field<Text>>,
    pub category: Option<crate::raw::Field<Text>>,
    pub iri: Option<crate::raw::Field<Text>>,
    pub discontinued: Option<crate::raw::Field<Text>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct Variation {
    pub variation_type: Option<crate::raw::Field<Text>>,
    pub mutation_type: Option<crate::raw::Field<Text>>,
    pub mutation_description: Option<crate::raw::Field<NativeText>>,
    pub variation_note: Option<crate::raw::Field<NativeText>>,
    pub zygosity_type: Option<crate::raw::Field<Text>>,
    pub xref_list: Option<crate::raw::Field<Vec<crate::raw::Field<Xref>>>>,
    pub source_list: Option<crate::raw::Field<Vec<crate::raw::Field<Provenance>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Provenance {
    pub reference: Option<crate::raw::Field<Reference>>,
    pub institution: Option<crate::raw::Field<NativeText>>,
    pub value: Option<crate::raw::Field<NativeText>>,
    pub xref: Option<crate::raw::Field<Xref>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Reference {
    #[serde(rename = "resource-internal-ref")]
    pub resource_internal_ref: Text,
}
#[derive(Debug, Deserialize)]
pub(super) struct Accession {
    #[serde(rename = "type")]
    pub kind: Text,
    pub value: CellosaurusAccession,
}
#[derive(Debug, Deserialize)]
pub(super) struct Name {
    #[serde(rename = "type")]
    pub kind: Text,
    #[serde(deserialize_with = "crate::native_text::deserialize")]
    pub value: biodata::Text,
}
#[derive(Debug, Deserialize)]
pub(super) struct Comment {
    pub category: Text,
    #[serde(deserialize_with = "crate::native_text::deserialize")]
    pub value: biodata::Text,
}
impl Record {
    pub(super) fn accession(&self) -> Result<&CellosaurusAccession, BadReply> {
        let mut rows = self
            .accession_list
            .iter()
            .filter_map(crate::raw::Field::as_ref)
            .filter(|a| a.kind.as_str() == "primary");
        let first = rows.next().ok_or(BadReply)?;
        if rows.next().is_some() {
            return Err(BadReply);
        }
        Ok(&first.value)
    }
    pub(super) fn name(&self) -> Option<&Text> {
        let mut rows = self
            .name_list
            .iter()
            .filter(|a| a.kind.as_str() == "identifier");
        let first = rows.next()?;
        if rows.next().is_some() {
            return None;
        }
        Some(&first.value)
    }
}
impl Reply for Response {
    fn check(&self) -> Result<(), BadReply> {
        match self {
            Self::Records { .. } => Ok(()),
            Self::Missing { code: 404, message }
                if message.starts_with("Item not found, ac: CVCL_") =>
            {
                Ok(())
            }
            _ => Err(BadReply),
        }
    }
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        match (self, status) {
            (Self::Records { .. }, 200) | (Self::Missing { code: 404, .. }, 404) => Ok(()),
            _ => Err(BadReply),
        }
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct ReleaseReply {
    #[serde(rename = "Cellosaurus")]
    pub data: ReleaseData,
}
#[derive(Debug, Deserialize)]
pub(super) struct ReleaseData {
    pub header: ReleaseHeader,
}
#[derive(Debug, Deserialize)]
pub(super) struct ReleaseHeader {
    pub release: Release,
}
impl Reply for ReleaseReply {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Release {
    pub version: Option<crate::raw::Field<biodata::Text>>,
    pub updated: Option<crate::raw::Field<biodata::PartialDate>>,
}
