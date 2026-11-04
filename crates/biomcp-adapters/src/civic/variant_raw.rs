use crate::{native_text::NativeText, raw::Field};
use biodata::Text;
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Profiles {
    pub data: Data,
    pub errors: Option<Vec<serde::de::IgnoredAny>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Data {
    pub molecular_profiles: Connection<Profile>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Connection<T> {
    pub total_count: u64,
    pub nodes: Vec<T>,
    pub page_info: Page,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Page {
    pub has_next_page: bool,
    pub end_cursor: Option<Text>,
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
    pub evidence_items: Connection<Evidence>,
    pub assertions: Connection<Assertion>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Member {
    pub id: Option<Field<u64>>,
    pub name: Option<Field<NativeText>>,
    pub link: Option<Field<Text>>,
    pub allele_registry_id: Option<Field<Text>>,
    pub hgvs_descriptions: Option<Field<Vec<Field<Text>>>>,
    pub feature: Option<Field<Feature>>,
}
#[derive(Deserialize)]
pub(super) struct Feature {
    pub id: Option<Field<u64>>,
    pub name: Option<Field<NativeText>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Evidence {
    pub id: Option<Field<u64>>,
    pub status: Option<Field<biomcp_types::CivicStatus>>,
    pub evidence_type: Option<Field<biodata::SourceText>>,
    pub evidence_level: Option<Field<Text>>,
    pub evidence_direction: Option<Field<Text>>,
    pub significance: Option<Field<biodata::SourceText>>,
    pub disease: Option<Field<Disease>>,
    pub therapies: Option<Field<Vec<Field<Therapy>>>>,
    pub therapy_interaction_type: Option<Field<Text>>,
    pub source: Option<Field<Citation>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Assertion {
    pub id: Option<Field<u64>>,
    pub status: Option<Field<biomcp_types::CivicStatus>>,
    pub assertion_type: Option<Field<biodata::SourceText>>,
    pub assertion_direction: Option<Field<Text>>,
    pub significance: Option<Field<biodata::SourceText>>,
    pub amp_level: Option<Field<Text>>,
    pub disease: Option<Field<Disease>>,
    pub therapies: Option<Field<Vec<Field<Therapy>>>>,
    pub therapy_interaction_type: Option<Field<Text>>,
    pub variant_origin: Option<Field<Text>>,
}
#[derive(Deserialize)]
pub(super) struct Disease {
    pub name: Option<Field<NativeText>>,
    pub doid: Option<Field<Text>>,
}
#[derive(Deserialize)]
pub(super) struct Therapy {
    pub id: Option<Field<u64>>,
    pub name: Option<Field<NativeText>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Citation {
    pub citation_id: Option<Field<Text>>,
    pub source_type: Option<Field<Text>>,
}
impl<T> Connection<T> {
    pub(super) fn progresses(&self, previous: Option<&Text>) -> bool {
        !self.page_info.has_next_page
            || !self.nodes.is_empty()
                && self
                    .page_info
                    .end_cursor
                    .as_ref()
                    .is_some_and(|cursor| Some(cursor) != previous)
    }
    pub(super) fn check(&self) -> Result<(), BadReply> {
        if self.nodes.len() as u64 > self.total_count
            || self.page_info.has_next_page
                && !self.nodes.is_empty()
                && self.page_info.end_cursor.is_none()
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
impl Profiles {
    pub(super) fn has_errors(&self) -> bool {
        self.errors
            .as_ref()
            .is_some_and(|errors| !errors.is_empty())
    }
    pub(super) fn progresses(
        &self,
        query: &biomcp_types::VariantGet,
        after: Option<&Text>,
    ) -> bool {
        use biomcp_types::CivicClaimKind;
        self.data.molecular_profiles.progresses(after)
            && self.data.molecular_profiles.nodes.iter().all(|profile| {
                (!query.evidence_filter().is_empty()
                    || query.civic_kind() == Some(CivicClaimKind::Assertions)
                    || profile.evidence_items.progresses(query.civic_cursor()))
                    && (query.civic_kind() == Some(CivicClaimKind::Evidence)
                        || profile.assertions.progresses(query.civic_cursor()))
            })
    }
    pub(super) fn matches(&self, allele: &str, profile: Option<u64>) -> bool {
        self.data.molecular_profiles.nodes.iter().all(|p| {
            profile.is_none_or(|id| p.id.as_ref().and_then(Field::as_ref) == Some(&id))
                && p.variants
                    .as_ref()
                    .and_then(Field::as_ref)
                    .is_some_and(|members| {
                        members.iter().filter_map(Field::as_ref).any(|m| {
                            m.id.as_ref()
                                .and_then(Field::as_ref)
                                .is_some_and(|value| id(*value))
                                && m.allele_registry_id
                                    .as_ref()
                                    .and_then(Field::as_ref)
                                    .is_some_and(|id| id.as_str() == allele)
                        })
                    })
        })
    }
}
impl Reply for Profiles {
    fn check(&self) -> Result<(), BadReply> {
        self.data.molecular_profiles.check()?;
        for profile in &self.data.molecular_profiles.nodes {
            profile.evidence_items.check()?;
            profile.assertions.check()?;
        }
        Ok(())
    }
}
pub(super) fn id(value: u64) -> bool {
    value > 0 && value <= i32::MAX as u64
}
