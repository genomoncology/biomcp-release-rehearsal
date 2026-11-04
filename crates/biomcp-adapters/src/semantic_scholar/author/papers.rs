use super::super::SPEC;
use crate::{mapped::Problems, raw::Field};
use biodata::SemanticScholarAuthorId;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::AuthorPapersQuery;
use serde::{Deserialize, Serialize};
mod row;
pub use row::Paper;
const COMPACT: &str = "paperId,externalIds,title,year,venue";
const DETAILS: &str = "paperId,externalIds,title,year,venue,abstract,publicationDate,citationCount,referenceCount,\
                    influentialCitationCount,isOpenAccess,openAccessPdf,fieldsOfStudy,publicationTypes,authors.authorId,authors.name";
/// Native papers page; the endpoint may omit both author identity and total.
#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "RawPage")]
pub struct PapersPage {
    /// Actual refused native records, independent of optional fields.
    #[serde(with = "crate::mapped::refusal_serde")]
    pub refusals: crate::Refusal,
    /// Native window start.
    pub offset: u32,
    /// Native next window, with no inferred continuation.
    pub next: Option<u32>,
    /// Native total, when supplied.
    pub total: Option<u64>,
    /// Unchanged native paper order.
    pub data: Vec<Paper>,
    /// Rejected independently identified records.
    pub problems: Vec<biomcp_types::Problem>,
    #[serde(skip)]
    pub(super) received: usize,
}
#[derive(Deserialize)]
struct RawPage {
    offset: u32,
    next: Option<u32>,
    total: Option<u64>,
    data: Vec<Field<Paper>>,
    problems: Option<Vec<biomcp_types::Problem>>,
    #[serde(
        default = "crate::mapped::refusal_serde::none",
        with = "crate::mapped::refusal_serde"
    )]
    refusals: crate::Refusal,
}
impl From<RawPage> for PapersPage {
    fn from(raw: RawPage) -> Self {
        let received = raw.data.len();
        let mut refusals = raw.refusals;
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: None,
            list: raw.problems.unwrap_or_default(),
        };
        let data = raw
            .data
            .into_iter()
            .filter_map(|r| {
                let row = p.record("paper_id", r, &mut refusals)?;
                refusals.include(row.refusals);
                Some(row)
            })
            .collect();
        Self {
            refusals,
            offset: raw.offset,
            next: raw.next,
            total: raw.total,
            data,
            problems: p.list,
            received,
        }
    }
}
impl Reply for PapersPage {
    fn check(&self) -> Result<(), BadReply> {
        for row in &self.data {
            row.clone().detail(&mut Vec::new())?;
        }
        if self.received > 25
            || self.next.is_some_and(|n| n <= self.offset)
            || self.total.is_some_and(|n| self.received as u64 > n)
        {
            Err(BadReply)
        } else {
            Ok(())
        }
    }
}
/// Fetch exactly the requested native window without enrichment.
pub fn papers(
    leg: &Leg<'_>,
    id: &SemanticScholarAuthorId,
    q: &AuthorPapersQuery,
) -> Result<PapersPage, RequestError> {
    let endpoint = SPEC
        .get("")
        .with_path(format!("/author/{id}/papers"))?
        .fresh(Freshness::Days(1));
    let fields = if q.section().is_some() {
        DETAILS
    } else {
        COMPACT
    };
    let (page, _) = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("fields", Arg::Text(fields)),
                ("offset", Arg::Num(u64::from(q.offset()))),
                ("limit", Arg::Num(u64::from(q.limit()))),
            ]),
            Read::<PapersPage>::json().check(&|page| {
                if page.offset == q.offset() && page.received <= q.limit() as usize {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(page)
}
