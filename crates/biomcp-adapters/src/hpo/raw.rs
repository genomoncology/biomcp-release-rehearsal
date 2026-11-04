use crate::raw::Field;
use biodata::{Curie, HpId};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Term {
    pub id: HpId,
    pub name: Option<Field<Box<str>>>,
    pub definition: Option<Field<Box<str>>>,
    pub comment: Option<Field<Box<str>>>,
    #[serde(rename = "publicationReferences")]
    pub publications: Option<Field<Vec<Field<Curie>>>>,
    #[serde(rename = "descendantCount")]
    pub descendants: Option<Field<u64>>,
    pub synonyms: Option<Field<Vec<Field<Box<str>>>>>,
    pub xrefs: Option<Field<Vec<Field<Curie>>>>,
}
impl Reply for Term {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
