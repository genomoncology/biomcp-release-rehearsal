//! Source-local therapy candidates; no chemical identity join is inferred.
use super::{GENES, variant::failure, variant_map, variant_raw::Connection};
use crate::{Mapped, mapped::Problems, native_text::NativeText, raw::Field};
use biodata::{DrugIdentityLevel, Text};
use biomcp_fetch::{BadReply, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{DrugHeader, DrugPage, DrugSource, Reason};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SearchReply {
    data: SearchData,
    errors: Option<Vec<serde::de::IgnoredAny>>,
}
#[derive(Deserialize)]
struct SearchData {
    therapies: Connection<Therapy>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Therapy {
    pub id: Option<Field<u64>>,
    pub name: Option<Field<NativeText>>,
    pub link: Option<Field<Text>>,
    pub therapy_aliases: Option<Field<Vec<Field<NativeText>>>>,
}
impl Therapy {
    fn search_header(self, cx: &mut Problems, text: &Text) -> Option<DrugHeader> {
        let exact_alias = self
            .therapy_aliases
            .as_ref()
            .and_then(Field::as_ref)
            .is_some_and(|names| {
                names.iter().filter_map(Field::as_ref).any(|name| {
                    biomcp_types::DrugNameMatch::same_name(name.as_str(), text.as_str())
                })
            });
        self.header(cx).map(|mut header| {
            if exact_alias {
                header.matched_name = Some(biomcp_types::DrugNameMatch::Generic);
            }
            header
        })
    }
    pub(super) fn header(self, cx: &mut Problems) -> Option<DrugHeader> {
        let id = variant_map::identity(self.id, "id", cx)?;
        cx.record = crate::mapped::record(id);
        let name = cx.source_field("name", self.name);
        let link = cx.field("link", self.link);
        let record = variant_map::record(id, link.as_ref(), cx)?;
        Some(DrugHeader {
            name_resolution: None,
            matched_name: None,
            id: cx.optional("id", Some(id.to_string().parse()))?,
            source: DrugSource::Civic,
            version: None,
            name,
            identity_level: DrugIdentityLevel::Unknown,
            r#type: cx.optional("type", Some(Text::new("Therapy"))),
            url: record.url,
        })
    }
}

impl SearchReply {
    fn cacheable(&self, count: u32, after: Option<&Text>) -> bool {
        self.data.therapies.nodes.len() <= count as usize
            && self.data.therapies.progresses(after)
            && self.data.therapies.nodes.iter().all(|row| {
                row.id
                    .as_ref()
                    .and_then(Field::as_ref)
                    .is_some_and(|id| super::variant_raw::id(*id))
            })
    }
}
impl Reply for SearchReply {
    fn check(&self) -> Result<(), BadReply> {
        if self.errors.as_ref().is_some_and(|rows| !rows.is_empty()) {
            return Err(BadReply);
        }
        self.data.therapies.check()?;
        Ok(())
    }
}
#[derive(Serialize)]
struct SearchRequest<'a> {
    query: String,
    variables: SearchVariables<'a>,
}
#[derive(Serialize)]
struct SearchVariables<'a> {
    q: &'a Text,
    after: Option<&'a Text>,
}
/// Search literal native therapy names in published order, without selecting a candidate.
/// Offset walking uses bounded source pages and checks cursor progress before requesting more.
pub fn drug_search(
    leg: &Leg<'_>,
    text: &Text,
    limit: u32,
    offset: u32,
) -> Result<Mapped<(DrugPage, u32)>, RequestError> {
    if !(1..=50).contains(&limit) {
        return Err(failure(Reason::BadReply));
    }
    let mut after = None;
    let mut skipped = 0;
    loop {
        let count = if skipped < offset {
            (offset - skipped).min(50)
        } else {
            limit
        };
        let body = SearchRequest {
            query: format!(
                "query($q:String!,$after:String){{therapies(name:$q,first:{count},after:$after){{totalCount \
                    pageInfo{{hasNextPage endCursor}} nodes{{id name link ncitId therapyAliases deprecated}}}}}}"
            ),
            variables: SearchVariables {
                q: text,
                after: after.as_ref(),
            },
        };
        let reply: SearchReply = leg
            .call(
                &GENES,
                Input::json(&[], &body),
                Read::json()
                    .cache_if(&|reply: &SearchReply| reply.cacheable(count, after.as_ref())),
            )?
            .value;
        let rows = reply.data.therapies;
        if rows.nodes.len() > count as usize || !rows.progresses(after.as_ref()) {
            return Err(failure(Reason::BadReply));
        }
        if skipped >= offset || !rows.page_info.has_next_page {
            let consumed = rows.nodes.len() as u32;
            let mut cx = Problems {
                source: biomcp_types::Source::Civic,
                record: None,
                list: Vec::new(),
            };
            let mut refusals = crate::Refusal::None;
            let items = if skipped < offset {
                Vec::new()
            } else {
                rows.nodes
                    .into_iter()
                    .filter_map(|row| {
                        cx.record = None;
                        match row.search_header(&mut cx, text) {
                            Some(row) => Some(row),
                            None => {
                                refusals = crate::Refusal::Refused;
                                None
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            };
            return Ok(Mapped {
                refusals,
                value: (
                    DrugPage {
                        items: items.into(),
                        total: Some(rows.total_count),
                    },
                    consumed,
                ),
                problems: cx.list,
            });
        }
        skipped = skipped
            .checked_add(rows.nodes.len() as u32)
            .ok_or_else(|| failure(Reason::TooLarge))?;
        after = rows.page_info.end_cursor;
    }
}
