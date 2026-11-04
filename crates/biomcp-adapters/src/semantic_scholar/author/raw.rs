use crate::native_text::NativeText;
use crate::{
    Mapped,
    mapped::{Problems, record},
    raw::Field,
};
use biodata::{
    AuthorCard, AuthorCountScope, AuthorCounts, AuthorHeader, AuthorId, SemanticScholarAuthorId,
    Text,
};
use biomcp_fetch::{BadReply, Reply};
use serde::{Deserialize, Serialize};
/// Native source page, with no inferred next offset or result total.
#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "RawPage")]
pub(super) struct Page {
    /// Native start offset.
    pub offset: u32,
    /// Native next offset, only when supplied.
    pub next: Option<u32>,
    /// Native matching records, when supplied.
    pub total: Option<u64>,
    /// Native candidate order.
    pub data: Vec<Record>,
    /// Rejected independently identified records.
    pub problems: Vec<biomcp_types::Problem>,
    #[serde(skip)]
    pub(super) received: usize,
    /// Actual refused native author records; never part of source serialization.
    #[serde(skip)]
    pub refusals: crate::Refusal,
}
#[derive(Deserialize)]
struct RawPage {
    offset: u32,
    next: Option<u32>,
    total: Option<u64>,
    data: Vec<Field<Record>>,
    problems: Option<Vec<biomcp_types::Problem>>,
}
impl From<RawPage> for Page {
    fn from(raw: RawPage) -> Self {
        let received = raw.data.len();
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: None,
            list: Vec::new(),
        };
        p.list.extend(raw.problems.into_iter().flatten());
        let mut refusals = crate::Refusal::None;
        let data = raw
            .data
            .into_iter()
            .filter_map(|r| match p.required("author_id", r.value()) {
                Some(row) => Some(row),
                None => {
                    refusals = crate::Refusal::Refused;
                    None
                }
            })
            .collect();
        Self {
            offset: raw.offset,
            next: raw.next,
            total: raw.total,
            data,
            problems: p.list,
            received,
            refusals,
        }
    }
}
/// Native author metadata; omitted optional fields remain absent.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", from = "RawRecord")]
pub(super) struct Record {
    /// Exact provider record identity.
    pub author_id: SemanticScholarAuthorId,
    name: Option<biodata::Text>,
    affiliations: Option<Box<[NativeText]>>,
    paper_count: Option<u64>,
    citation_count: Option<u64>,
    h_index: Option<u32>,
    url: Option<Text>,
    problems: Vec<biomcp_types::Problem>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRecord {
    author_id: SemanticScholarAuthorId,
    name: Option<Field<NativeText>>,
    affiliations: Option<Field<Vec<Field<NativeText>>>>,
    paper_count: Option<Field<u64>>,
    citation_count: Option<Field<u64>>,
    h_index: Option<Field<u32>>,
    url: Option<Field<Text>>,
    problems: Option<Vec<biomcp_types::Problem>>,
}
impl From<RawRecord> for Record {
    fn from(raw: RawRecord) -> Self {
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: record(&raw.author_id),
            list: Vec::new(),
        };
        p.list.extend(raw.problems.into_iter().flatten());
        let affiliations = p.field("affiliations", raw.affiliations).map(|rows| {
            rows.into_iter()
                .filter_map(|v| p.field("affiliations", Some(v)))
                .collect()
        });
        Self {
            name: p.source_field("name", raw.name),
            affiliations,
            paper_count: p.field("paper_count", raw.paper_count),
            citation_count: p.field("citation_count", raw.citation_count),
            h_index: p.field("h_index", raw.h_index),
            url: p.field("url", raw.url),
            author_id: raw.author_id,
            problems: p.list,
        }
    }
}
impl Record {
    /// Preserve optional metadata and its provider-record count scope.
    pub(super) fn header(self, problems: &mut Vec<biomcp_types::Problem>) -> AuthorHeader {
        problems.extend(self.problems);
        let counts =
            (self.paper_count.is_some() || self.citation_count.is_some() || self.h_index.is_some())
                .then_some(AuthorCounts {
                    scope: AuthorCountScope::ProviderRecord,
                    paper_count: self.paper_count,
                    citation_count: self.citation_count,
                    h_index: self.h_index,
                });
        AuthorHeader {
            id: AuthorId::SemanticScholar(self.author_id),
            name: self.name,
            affiliations: self
                .affiliations
                .map(|values| values.into_vec().into_iter().map(Into::into).collect()),
            counts,
        }
    }
    /// Keep the provider link separate from the candidate metadata.
    pub(super) fn card(self) -> Mapped<AuthorCard> {
        let mut problems = Vec::new();
        let url = self.url.clone();
        let value = AuthorCard {
            header: self.header(&mut problems),
            url,
        };
        Mapped {
            refusals: crate::Refusal::None,
            value,
            problems,
        }
    }
}
impl Reply for Record {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.received > 25
            || self.next.is_some_and(|n| n <= self.offset)
            || self.total.is_some_and(|t| self.received as u64 > t)
        {
            Err(BadReply)
        } else {
            Ok(())
        }
    }
}
