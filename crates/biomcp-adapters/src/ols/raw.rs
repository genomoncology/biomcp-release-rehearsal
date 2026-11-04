use crate::native_text::NativeText;
use crate::raw::Field;
use biodata::{Curie, Text};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct Search {
    pub response: SearchResponse,
}
#[derive(Deserialize)]
pub(super) struct SearchResponse {
    #[serde(rename = "numFound")]
    pub total: u64,
    pub start: u32,
    pub docs: Vec<Hit>,
}
#[derive(Deserialize)]
pub(super) struct Hit {
    pub obo_id: Option<Field<Curie>>,
    pub label: Option<Field<NativeText>>,
    pub ontology_name: String,
    pub iri: Option<Field<Text>>,
    pub is_obsolete: Option<crate::raw::Field<bool>>,
}
impl Reply for Search {
    fn check(&self) -> Result<(), BadReply> {
        if self.response.docs.len() > 50 || self.response.docs.len() as u64 > self.response.total {
            return Err(BadReply);
        }
        Ok(())
    }
}
#[derive(Deserialize)]
pub(super) struct Exact {
    #[serde(rename = "_embedded")]
    pub embedded: Embedded,
}
#[derive(Deserialize)]
pub(super) struct Embedded {
    pub terms: Vec<Term>,
}
#[derive(Deserialize)]
pub(super) struct Term {
    pub obo_id: Curie,
    pub label: Option<Field<NativeText>>,
    pub ontology_name: String,
    pub iri: Option<Field<Text>>,
    pub description: Option<Field<Vec<Field<NativeText>>>>,
    pub is_obsolete: Option<crate::raw::Field<bool>>,
    pub term_replaced_by: Option<Field<Vec<Field<String>>>>,
    pub annotation: Option<Field<Annotation>>,
    pub synonyms: Option<Field<Vec<Field<NativeText>>>>,
    pub obo_synonym: Option<Field<Vec<Field<Synonym>>>>,
    pub obo_xref: Option<Field<Vec<Field<Xref>>>>,
}
#[derive(Deserialize)]
pub(super) struct Annotation {
    pub consider: Option<Field<Vec<Field<String>>>>,
}
#[derive(Deserialize)]
pub(super) struct Synonym {
    pub name: Field<NativeText>,
    pub scope: Option<Field<String>>,
}
#[derive(Deserialize)]
pub(super) struct Xref {
    pub database: Field<String>,
    pub id: Field<String>,
    pub description: Option<Field<NativeText>>,
}
impl Reply for Exact {
    fn check(&self) -> Result<(), BadReply> {
        if self.embedded.terms.len() > 1 {
            return Err(BadReply);
        }
        Ok(())
    }
}
