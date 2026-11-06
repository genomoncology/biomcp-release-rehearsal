//! PubMed native search order and citation metadata, without abstracts.
use biomcp_fetch::{Input, Read};
mod efetch;
mod indexing;
mod map;
pub use efetch::{FetchedArticle, fetch_articles, resolve_ids};
pub(crate) use map::publication_type;
mod raw;
use crate::{Mapped, quotas};
use biodata::Pmid;

use biomcp_fetch::{
    Arg, Credential, Endpoint, Freshness, KeyPlace, KeyUse, Leg, RequestError, Secret, SecretName,
    SourceSpec,
};
use biomcp_types::ArticleHeader;
use biomcp_types::{ArticleSort, Source, SourceNotice};
use std::time::Duration;
/// Shared NCBI E-utilities policy.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "NCBI policies; publisher rights in abstracts",
        url: "https://www.ncbi.nlm.nih.gov/home/about/policies/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::PubMed,
    base: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils",
    key: KeyUse::Optional(Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::NcbiApiKey,
        place: KeyPlace::Query("api_key"),
    }),
    quota: &quotas::NCBI,
    deadline: Duration::from_secs(15),
};
const SEARCH: Endpoint = SPEC.get("/esearch.fcgi").fresh(Freshness::Days(1));
// https://www.ncbi.nlm.nih.gov/books/NBK25499/ recommends POST for long ESearch terms.
// Keep conservative GET headroom without reimplementing request percent encoding.
const MAX_GET_TERM_BYTES: usize = 1000;
const SEARCH_POST: Endpoint = SPEC
    .post_form("/esearch.fcgi")
    .idempotent()
    .fresh(Freshness::Days(1));
const SUMMARY: Endpoint = SPEC
    .post_form("/esummary.fcgi")
    .idempotent()
    .fresh(Freshness::Days(1));
/// PubMed's original full requested window and query interpretation.
#[derive(Debug)]
pub struct SearchIds {
    /// Ordered unique checked identities.
    pub ids: Vec<Pmid>,
    /// All matching PubMed records.
    pub count: u64,
    /// PubMed words it could not find.
    pub notices: Vec<SourceNotice>,
    /// Rejected optional notice fields.
    pub problems: Vec<biomcp_types::Problem>,
}
/// Retrieve the application's fixed requested window in the requested native provider order.
pub fn search_ids(
    leg: &Leg<'_>,
    term: &str,
    sort: ArticleSort,
    email: Option<&Secret>,
) -> Result<SearchIds, RequestError> {
    search_ids_with_inputs(leg, term, sort, email, None)
}
/// Preserve the explicit identifier group that built a search term.
pub fn search_ids_with_inputs(
    leg: &Leg<'_>,
    term: &str,
    sort: ArticleSort,
    email: Option<&Secret>,
    inputs: Option<biomcp_fetch::Identities>,
) -> Result<SearchIds, RequestError> {
    let mut params = vec![
        ("db", Arg::Text("pubmed")),
        ("term", Arg::Text(term)),
        ("retmax", Arg::Num(9999)),
        ("retmode", Arg::Text("json")),
        ("tool", Arg::Text("biomcp")),
    ];
    if sort != ArticleSort::Date {
        params.insert(2, ("sort", Arg::Text("relevance")));
    }
    if let Some(email) = email {
        params.push(("email", Arg::Contact(email)));
    }
    let endpoint = if term.len() > MAX_GET_TERM_BYTES {
        &SEARCH_POST
    } else {
        &SEARCH
    };
    let raw = match inputs {
        Some(ids) => leg
            .call(
                endpoint,
                Input::Params(&params),
                Read::<raw::Search>::json().inputs(ids, "term"),
            )
            .map(|received| received.value),
        None => leg
            .call(
                endpoint,
                Input::Params(&params),
                Read::<raw::Search>::json(),
            )
            .map(|received| received.value),
    }?
    .esearchresult;
    Ok(search_result(raw))
}
fn search_result(raw: raw::SearchResult) -> SearchIds {
    let mut problems = crate::mapped::Problems {
        source: Source::PubMed,
        record: None,
        list: Vec::new(),
    };
    let mut notices = Vec::new();
    if let Some(errors) = raw
        .errorlist
        .and_then(|v| problems.required("notices.not_found_words", v.value()))
        && let Some(words) = errors
            .phrasesnotfound
            .and_then(|v| problems.required("notices.not_found_words", v.value()))
    {
        notices.push(SourceNotice::NotFoundWords {
            words: words
                .into_iter()
                .filter_map(|v| {
                    let s = problems.required("notices.not_found_words", v.value())?;
                    problems.text("notices.not_found_words", Some(&s))
                })
                .collect(),
        });
    }
    if let Some(warnings) = raw
        .warninglist
        .and_then(|v| problems.required("notices.pubmed_warnings", v.value()))
    {
        for (name, phrases) in warnings {
            if let Some(phrases) =
                problems.required("notices.pubmed_warning.phrases", phrases.value())
            {
                notices.push(SourceNotice::PubmedWarning {
                    name,
                    phrases: phrases
                        .into_iter()
                        .filter_map(|v| {
                            problems.required("notices.pubmed_warning.phrases", v.value())
                        })
                        .collect(),
                });
            }
        }
    }
    SearchIds {
        ids: raw.idlist,
        count: raw.count,
        notices,
        problems: problems.list,
    }
}
/// Retrieve a selected page's metadata. Each result is joined by checked PMID.
pub fn summaries(
    leg: &Leg<'_>,
    ids: &[Pmid],
    email: Option<&Secret>,
) -> Result<Mapped<Vec<ArticleHeader>>, RequestError> {
    summaries_reply(leg, ids, email).map(|(reply, ids)| map::summaries(reply, &ids))
}
/// Citation author assertions, preserving each paper independently.
pub fn author_summaries(
    leg: &Leg<'_>,
    ids: &[Pmid],
    email: Option<&Secret>,
) -> Result<Mapped<Vec<biomcp_types::AuthorPublication>>, RequestError> {
    let (reply, ids) = summaries_reply(leg, ids, email)?;
    Ok(map::author_summaries(reply, &ids))
}
fn summaries_reply(
    leg: &Leg<'_>,
    ids: &[Pmid],
    email: Option<&Secret>,
) -> Result<(raw::Summary, Vec<Box<str>>), RequestError> {
    let ids: Vec<_> = ids
        .iter()
        .map(|id| id.to_string().into_boxed_str())
        .collect();
    let mut params = vec![
        ("db", Arg::Text("pubmed")),
        ("id", Arg::Ids(&ids)),
        ("retmode", Arg::Text("json")),
        ("tool", Arg::Text("biomcp")),
    ];
    if let Some(email) = email {
        params.push(("email", Arg::Contact(email)));
    }
    let reply = leg
        .call(
            &SUMMARY,
            Input::Params(&params),
            Read::<raw::Summary>::json(),
        )?
        .value;
    Ok((reply, ids))
}

mod query;
pub use query::{search_dois, search_query};

/// Complete source policy and command declaration.
pub(crate) mod decl;
