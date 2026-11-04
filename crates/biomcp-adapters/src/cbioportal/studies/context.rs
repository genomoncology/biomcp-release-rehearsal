//! Permission and resource ownership remain whole-document admission checks.
use biodata::StudyId;
use biomcp_fetch::BadReply;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Context {
    study_id: Option<crate::raw::Field<StudyId>>,
    #[serde(rename = "groups")]
    _groups: Box<str>,
    #[serde(rename = "readPermission")]
    _read_permission: bool,
    #[serde(rename = "publicStudy")]
    _public_study: bool,
    resource_counts: Vec<Resource>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Resource {
    cancer_study_identifier: StudyId,
    #[serde(rename = "openByDefault")]
    _open_by_default: bool,
}
impl Context {
    pub(super) fn check(&self) -> Result<(), BadReply> {
        if let Some(id) = self.study_id.as_ref().and_then(crate::raw::Field::as_ref)
            && self
                .resource_counts
                .iter()
                .any(|r| &r.cancer_study_identifier != id)
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
