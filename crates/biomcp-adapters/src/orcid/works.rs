use super::raw::{Wrapped, optional};
use crate::{
    Mapped,
    mapped::{Problems, record},
    raw::Field,
};
use biodata::{
    OrcidAssertionSource, OrcidClientId, OrcidExternalId, OrcidId, OrcidPublicVisibility,
    OrcidWorkAssertion, OrcidWorkGroup, PartialDate, Text,
};
use biomcp_fetch::{BadReply, Reply};
use biomcp_types::Source;
use serde::Deserialize;
use std::num::NonZeroU64;
#[derive(Deserialize)]
pub(super) struct Works {
    pub path: String,
    group: Vec<Field<Group>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Group {
    external_ids: Option<Field<Identifiers>>,
    work_summary: Option<Field<Vec<Field<Assertion>>>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Identifiers {
    external_id: Vec<Field<ExternalId>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ExternalId {
    external_id_type: Option<Field<Text>>,
    external_id_value: Text,
    external_id_relationship: Option<Field<Text>>,
    external_id_url: Option<Field<Url>>,
}
#[derive(Deserialize)]
struct Url {
    value: Text,
}
impl Identifiers {
    fn convert(self, problems: &mut Problems) -> Box<[OrcidExternalId]> {
        self.external_id
            .into_iter()
            .filter_map(|id| {
                let id = problems.required("external_ids", id.value())?;
                Some(OrcidExternalId {
                    namespace: problems.field("external_ids", id.external_id_type).into(),
                    id: id.external_id_value,
                    relationship: optional(
                        problems,
                        "external_ids.relationship",
                        id.external_id_relationship,
                    ),
                    url: optional(problems, "external_ids.url", id.external_id_url)
                        .map(|v| v.value),
                })
            })
            .collect()
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Assertion {
    path: Option<Field<String>>,
    put_code: Option<Field<NonZeroU64>>,
    visibility: Option<String>,
    title: Option<Field<Title>>,
    #[serde(rename = "type")]
    work_type: Option<Field<Text>>,
    publication_date: Option<Field<Date>>,
    journal_title: Option<Field<Wrapped>>,
    url: Option<Field<Url>>,
    external_ids: Option<Field<Identifiers>>,
    source: Option<Field<Origin>>,
    display_index: Option<Field<Text>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Title {
    title: Option<Field<Wrapped>>,
    subtitle: Option<Field<Wrapped>>,
    translated_title: Option<Field<TranslatedTitle>>,
}
#[derive(Deserialize)]
struct TranslatedTitle {
    #[serde(deserialize_with = "crate::native_text::deserialize")]
    content: biodata::Text,
}
#[derive(Deserialize)]
struct Date {
    year: DatePart,
    month: Option<DatePart>,
    day: Option<DatePart>,
}
#[derive(Deserialize)]
struct DatePart {
    value: Text,
}
impl Date {
    fn convert(&self) -> Result<PartialDate, BadReply> {
        PartialDate::new(
            self.year.value.as_str().parse().map_err(|_| BadReply)?,
            self.month
                .as_ref()
                .map(|v| v.value.as_str().parse())
                .transpose()
                .map_err(|_| BadReply)?,
            self.day
                .as_ref()
                .map(|v| v.value.as_str().parse())
                .transpose()
                .map_err(|_| BadReply)?,
        )
        .map_err(|_| BadReply)
    }
}
#[derive(Deserialize)]
struct Identity<T> {
    path: T,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Origin {
    source_orcid: Option<Field<Identity<OrcidId>>>,
    source_client_id: Option<Field<Identity<OrcidClientId>>>,
    source_name: Option<Field<Wrapped>>,
    assertion_origin_orcid: Option<Field<Identity<OrcidId>>>,
    assertion_origin_client_id: Option<Field<Identity<OrcidClientId>>>,
    assertion_origin_name: Option<Field<Wrapped>>,
}
impl Origin {
    fn convert(self, problems: &mut Problems) -> OrcidAssertionSource {
        OrcidAssertionSource {
            source_orcid: optional(problems, "source.source_orcid", self.source_orcid)
                .map(|v| v.path),
            source_client_id: optional(problems, "source.source_client_id", self.source_client_id)
                .map(|v| v.path),
            source_name: optional(problems, "source.source_name", self.source_name)
                .map(|v| v.value),
            assertion_origin_orcid: optional(
                problems,
                "source.assertion_origin_orcid",
                self.assertion_origin_orcid,
            )
            .map(|v| v.path),
            assertion_origin_client_id: optional(
                problems,
                "source.assertion_origin_client_id",
                self.assertion_origin_client_id,
            )
            .map(|v| v.path),
            assertion_origin_name: optional(
                problems,
                "source.assertion_origin_name",
                self.assertion_origin_name,
            )
            .map(|v| v.value),
        }
    }
}
impl Assertion {
    fn convert(mut self, prefix: &str, problems: &mut Problems) -> Option<OrcidWorkAssertion> {
        let path = problems.required_field("path", self.path.take())?;
        problems.record = record(&path);
        let put_code = problems.required_field("put_code", self.put_code.take())?;
        if path != format!("{prefix}/work/{put_code}") {
            problems.required::<()>("path", None);
            return None;
        }
        let (title, subtitle, translated_title) = optional(problems, "title", self.title)
            .map(|v| {
                (
                    optional(problems, "title", v.title).map(|v| v.value),
                    optional(problems, "subtitle", v.subtitle).map(|v| v.value),
                    optional(problems, "translated_title", v.translated_title).map(|v| v.content),
                )
            })
            .unwrap_or_default();
        Some(OrcidWorkAssertion {
            put_code,
            title,
            subtitle,
            translated_title,
            work_type: optional(problems, "work_type", self.work_type),
            publication_date: optional(problems, "publication_date", self.publication_date)
                .and_then(|v| problems.optional("publication_date", Some(v.convert()))),
            journal_title: optional(problems, "journal_title", self.journal_title).map(|v| v.value),
            url: optional(problems, "url", self.url).map(|v| v.value),
            external_ids: optional(problems, "external_ids", self.external_ids)
                .map(|v| v.convert(problems))
                .unwrap_or_default(),
            source: optional(problems, "source", self.source).map(|v| v.convert(problems)),
            visibility: OrcidPublicVisibility::Public,
            display_index: optional(problems, "display_index", self.display_index),
        })
    }
}
impl Works {
    pub(super) fn identity(&self, id: &OrcidId) -> bool {
        self.path == format!("/{id}/works")
    }
    pub(super) fn groups(self) -> Mapped<Box<[OrcidWorkGroup]>> {
        let mut problems = Problems {
            source: Source::Orcid,
            record: record(&self.path),
            list: Vec::new(),
        };
        let prefix = self.path.strip_suffix("/works").unwrap_or("");
        let mut refusals = crate::Refusal::None;
        let value = self
            .group
            .into_iter()
            .filter_map(|g| {
                problems.record = record(&self.path);
                let Some(g) = problems.field("group", Some(g)) else {
                    refusals = crate::Refusal::Refused;
                    return None;
                };
                let summaries = problems
                    .field("work_summary", g.work_summary)
                    .unwrap_or_default();
                let assertions: Box<[_]> = summaries
                    .into_iter()
                    .filter_map(|s| {
                        problems.record = record(&self.path);
                        let Some(s) = problems.field("work_summary", Some(s)) else {
                            refusals = crate::Refusal::Refused;
                            return None;
                        };
                        if s.visibility.as_deref() != Some("public") {
                            return None;
                        }
                        match s.convert(prefix, &mut problems) {
                            Some(row) => Some(row),
                            None => {
                                refusals = crate::Refusal::Refused;
                                None
                            }
                        }
                    })
                    .collect();
                if assertions.is_empty() {
                    None
                } else {
                    problems.record = record(&self.path);
                    let external_ids = optional(&mut problems, "external_ids", g.external_ids)
                        .map(|v| v.convert(&mut problems))
                        .unwrap_or_default();
                    Some(OrcidWorkGroup {
                        external_ids,
                        assertions,
                    })
                }
            })
            .collect();
        Mapped {
            refusals,
            value,
            problems: problems.list,
        }
    }
}
impl Reply for Works {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
