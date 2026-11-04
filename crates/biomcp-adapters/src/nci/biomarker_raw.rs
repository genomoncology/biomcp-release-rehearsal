//! Complete native biomarker observations reject cut collections and source errors.
use crate::native_text::NativeText;
use crate::raw::Field;
use biodata::{NcitId, Text};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Page {
    pub total: u64,
    error: Option<serde::de::IgnoredAny>,
    pub data: Vec<Biomarker>,
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.error.is_some()
            || self.total != self.data.len() as u64
            || self.data.iter().any(|row| row.codes.is_empty())
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Biomarker {
    pub name: Option<Field<NativeText>>,
    pub codes: Box<[NcitId]>,
    pub r#type: Option<Field<Vec<Field<Text>>>>,
    pub count: Option<Field<u64>>,
    pub parent_ids: Option<Field<Vec<Field<NcitId>>>>,
    pub ancestor_ids: Option<Field<Vec<Field<NcitId>>>>,
    pub synonyms: Option<Field<Vec<Field<NativeText>>>>,
    pub eligibility_criterion: Option<Field<Vec<Field<Text>>>>,
}
