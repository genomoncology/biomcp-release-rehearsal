use super::{drug::Therapy, variant_raw as raw};
use crate::{native_text::NativeText, raw::Field};
use biodata::Text;
use biomcp_fetch::{BadReply, Reply};
use biomcp_types::DrugGet;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct ReplyBody {
    pub data: Data,
    pub errors: Option<Vec<serde::de::IgnoredAny>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Data {
    pub therapy: Option<Therapy>,
    pub evidence_items: Option<raw::Connection<Evidence>>,
    pub assertions: Option<raw::Connection<Assertion>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Profile {
    pub id: Option<Field<u64>>,
    pub name: Option<Field<NativeText>>,
    pub raw_name: Option<Field<Text>>,
    pub link: Option<Field<Text>>,
    pub is_complex: Option<Field<bool>>,
    pub variants: Option<Field<Vec<Field<Member>>>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Evidence {
    #[serde(flatten)]
    pub claim: raw::Evidence,
    pub molecular_profile: Option<Field<Profile>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Assertion {
    #[serde(flatten)]
    pub claim: raw::Assertion,
    pub molecular_profile: Option<Field<Profile>>,
}
impl Reply for ReplyBody {
    fn check(&self) -> Result<(), BadReply> {
        if self.errors.as_ref().is_some_and(|rows| !rows.is_empty()) {
            return Err(BadReply);
        }
        if let Some(rows) = &self.data.evidence_items {
            rows.check()?;
        }
        if let Some(rows) = &self.data.assertions {
            rows.check()?;
        }
        Ok(())
    }
}
#[derive(Deserialize)]
pub(super) struct Member {
    #[serde(flatten)]
    pub variant: raw::Member,
    pub coordinates: Option<Field<Coordinates>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Coordinates {
    pub reference_build: Option<Field<Text>>,
}

pub(super) fn cacheable(data: &Data, query: &DrugGet) -> bool {
    use crate::raw::Field;
    let selected = |profile: &Option<Field<Profile>>,
                    id: &Option<Field<u64>>,
                    therapies: &Option<Field<Vec<Field<raw::Therapy>>>>| {
        id.as_ref()
            .and_then(Field::as_ref)
            .is_some_and(|id| raw::id(*id))
            && profile
                .as_ref()
                .and_then(Field::as_ref)
                .is_some_and(|profile| {
                    query
                        .civic_profile()
                        .is_none_or(|id| profile.id.as_ref().and_then(Field::as_ref) == Some(&id))
                })
            && therapies
                .as_ref()
                .and_then(Field::as_ref)
                .is_some_and(|rows| {
                    rows.iter().filter_map(Field::as_ref).any(|t| {
                        t.id.as_ref()
                            .and_then(Field::as_ref)
                            .is_some_and(|id| id.to_string() == query.input().to_string())
                    })
                })
    };
    data.therapy
        .as_ref()
        .and_then(|t| t.id.as_ref())
        .and_then(Field::as_ref)
        .is_some_and(|id| id.to_string() == query.input().to_string())
        && data.evidence_items.as_ref().is_none_or(|rows| {
            rows.nodes
                .iter()
                .all(|row| selected(&row.molecular_profile, &row.claim.id, &row.claim.therapies))
        })
        && data.assertions.as_ref().is_none_or(|rows| {
            rows.nodes
                .iter()
                .all(|row| selected(&row.molecular_profile, &row.claim.id, &row.claim.therapies))
        })
}
