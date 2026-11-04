//! Provider-specific article terms and native year intervals.
use super::search::{Search, search};
use biomcp_fetch::{Leg, RequestError};
/// Search checked article selectors using the provider's term and year syntax.
pub fn search_query(
    leg: &Leg<'_>,
    query: &biomcp_types::ArticleSearch,
) -> (Result<Search, RequestError>, String) {
    let term = query
        .gene()
        .map_or_else(|| query.words(), |gene| format!("{gene} {}", query.words()))
        .trim()
        .to_owned();
    let (since, until) = query.dates();
    let year = (since.is_some() || until.is_some()).then(|| {
        format!(
            "{}-{}",
            since.map_or_else(String::new, |v| v.year().to_string()),
            until.map_or_else(String::new, |v| v.year().to_string())
        )
    });
    (
        search(leg, &term, year.as_deref(), query.open_access()),
        term,
    )
}
