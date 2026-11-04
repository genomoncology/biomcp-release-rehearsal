use crate::raw::Field;
use biodata::Pmid;
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct Export {
    #[serde(rename = "PubTator3")]
    pub documents: Vec<Field<Document>>,
}
#[derive(Deserialize)]
pub(super) struct Document {
    pub id: Pmid,
    pub passages: Option<Field<Vec<Field<Passage>>>>,
}
#[derive(Deserialize)]
pub(super) struct Passage {
    pub annotations: Option<Field<Vec<Field<Annotation>>>>,
}
#[derive(Deserialize)]
pub(super) struct Annotation {
    pub infons: Infons,
    pub text: Option<Field<String>>,
}
#[derive(Deserialize)]
pub(super) struct Infons {
    #[serde(rename = "type")]
    pub kind: String,
    pub identifier: Option<Field<String>>,
    pub name: Option<Field<String>>,
    pub hgvs: Option<Field<String>>,
    pub rsid: Option<Field<String>>,
    pub gene_id: Option<Field<u32>>,
}
impl biomcp_fetch::Reply for Export {
    fn check(&self) -> Result<(), biomcp_fetch::BadReply> {
        let mut ids = std::collections::BTreeSet::new();
        if self
            .documents
            .iter()
            .filter_map(Field::as_ref)
            .any(|d| !ids.insert(d.id))
        {
            return Err(biomcp_fetch::BadReply);
        }
        Ok(())
    }
}
