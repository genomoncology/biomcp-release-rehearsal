//! Literal native author candidates and request-checked exact provider cards.
use biomcp_fetch::Input;
mod papers;
mod raw;
use super::SPEC;
use biodata::{AuthorCard, SemanticScholarAuthorId};
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError};
use biomcp_types::AuthorSearch;
pub use papers::{Paper, PapersPage, papers};
mod map;
pub use map::Page;
const FIELDS: &str = "authorId,name,affiliations,paperCount,citationCount,hIndex,url";
/// Read one unchanged source query in native order.
pub fn search(leg: &Leg<'_>, q: &AuthorSearch) -> Result<Page, RequestError> {
    let endpoint = SPEC.get("/author/search").fresh(Freshness::Days(1));
    let (page, _) = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("query", Arg::Text(q.query())),
                ("fields", Arg::Text(FIELDS)),
                ("offset", Arg::Num(u64::from(q.offset()))),
                ("limit", Arg::Num(u64::from(q.limit()))),
            ]),
            biomcp_fetch::Read::<raw::Page>::json().check(&|page| {
                if page.offset == q.offset() && page.received <= q.limit() as usize {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(map::Page::new(page))
}
/// Reject a mismatched returned identity before cache admission and on cache reads.
pub fn card(
    leg: &Leg<'_>,
    id: &SemanticScholarAuthorId,
) -> Result<crate::Mapped<AuthorCard>, RequestError> {
    let endpoint = SPEC
        .get("")
        .with_path(format!("/author/{id}"))?
        .fresh(Freshness::Days(1));
    let (row, _) = leg
        .call(
            &endpoint,
            Input::Params(&[("fields", Arg::Text(FIELDS))]),
            biomcp_fetch::Read::<raw::Record>::json().check(&|row| {
                if &row.author_id == id {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok(row.card())
}
