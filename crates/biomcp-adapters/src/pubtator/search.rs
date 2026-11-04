//! Bounded PubTator3 native pages; no autocomplete or local score ordering.
use super::SPEC;
use biodata::Pmid;
use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::Source;
use serde::Deserialize;
const SEARCH: Endpoint = SPEC.get("/search/").fresh(Freshness::Days(1));
const LIMIT: usize = 100;
/// A retained prefix and any failure that prevented completing its bounded window.
#[derive(Debug)]
pub struct Search {
    /// Native rows before the shared fold removes duplicate papers.
    pub ids: Vec<Pmid>,
    /// Provider's total, separate from our fetched prefix.
    pub count: u64,
    /// A later-page failure never discards earlier successful pages.
    pub error: Option<RequestError>,
}
/// Fetch until the native-row bound, terminal pagination, repeated page or typed failure.
pub fn search(leg: &Leg<'_>, term: &str) -> Result<Search, RequestError> {
    let mut result = Search {
        ids: Vec::new(),
        count: 0,
        error: None,
    };
    let mut previous = Vec::new();
    for page in 1..=LIMIT as u64 {
        let reply = leg
            .call(
                &SEARCH,
                Input::Params(&[("text", Arg::Text(term)), ("page", Arg::Num(page))]),
                Read::<Page>::json(),
            )
            .map(|received| received.value);
        let reply = match reply {
            Ok(reply) => reply,
            Err(error) if page == 1 => return Err(error),
            Err(error) => {
                result.error = Some(error);
                break;
            }
        };
        if page == 1 {
            result.count = reply.count;
        }
        let ids: Vec<_> = reply.results.into_iter().map(|r| r.pmid.0).collect();
        if reply.current != page
            || !ids.is_empty() && ids == previous
            || reply.count != result.count
        {
            result.error = Some(biomcp_fetch::bad_reply(Source::PubTator, None));
            break;
        }
        let terminal = ids.is_empty() || page >= reply.total_pages;
        for id in &ids {
            if result.ids.len() < LIMIT {
                result.ids.push(*id);
            }
        }
        previous = ids;
        if terminal || result.ids.len() >= LIMIT {
            break;
        }
        if page == LIMIT as u64 {
            result.error = Some(biomcp_fetch::bad_reply(Source::PubTator, None));
        }
    }
    Ok(result)
}

#[derive(Deserialize)]
struct Page {
    results: Vec<Row>,
    count: u64,
    current: u64,
    total_pages: u64,
    page_size: u64,
}
#[derive(Deserialize)]
struct Row {
    pmid: crate::raw::NumericPmid,
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.page_size == 0
            || self.page_size > 100
            || self.results.len() as u64 > self.page_size
            || self.current == 0
            || self.results.len() as u64 > self.count
        {
            return Err(BadReply);
        }
        Ok(())
    }
}

/// Compose the native term for checked gene and literal article selectors.
pub fn query_term(
    query: &biomcp_types::ArticleSearch,
    entrez: Option<biodata::NcbiGeneId>,
) -> String {
    entrez
        .map(|id| format!("@GENE_{id}"))
        .into_iter()
        .chain([query.words()])
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" AND ")
}
