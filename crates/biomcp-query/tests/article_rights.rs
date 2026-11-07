//! Restricted article text is filtered before raw and mapped persistence.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    reason = "Isolated synthetic transport and SQLite witness"
)]
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use biomcp_types::{ArticleSearch, ReplyData, Request, Use};
use std::time::Duration;

fn run(fetch: &Fetch) -> biomcp_query::Outcome {
    biomcp_query::run(
        &biomcp_query::Ctx {
            fetch,
            budget: Duration::from_secs(20),
            fresh: false,
            ncbi_email: None,
            drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
        },
        &Request::SearchArticle(
            serde_json::from_str::<ArticleSearch>(
                r#"{"text":"rights witness","exact_term":true,"source":["litsense"],"limit":3}"#,
            )
            .unwrap(),
        ),
    )
}
fn replay(path: &std::path::Path) -> Fetch {
    Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: path.into() },
        cache_owners: vec![biomcp_query::store::OWNER],
        ..FetchConfig::default()
    })
    .unwrap()
}
fn hosted(path: &std::path::Path) -> Fetch {
    replay(path)
        .restrict_uses([Use::PerRecord].into())
        .with_hosted_secrets(biomcp_fetch::Secrets::default())
}
#[path = "article_rights/fixture.rs"]
mod fixture;
use fixture::record;

#[test]
fn restricted_keeps_every_paper_and_persists_only_permitted_body_text() {
    let input = tempfile::tempdir().unwrap();
    record(input.path());
    let fetch = hosted(input.path());
    let out = run(&fetch);
    let ReplyData::ArticlePage {
        body: Some(page), ..
    } = out.body
    else {
        panic!("{out:?}")
    };
    assert_eq!(
        page.items
            .iter()
            .filter_map(|row| row.pmid)
            .map(|id| id.to_string())
            .collect::<Vec<_>>(),
        ["1001", "1002", "1003"],
        "text admission must preserve every paper in provider order"
    );
    assert_eq!(
        page.items
            .iter()
            .filter(|row| row.sentence.is_some())
            .count(),
        1,
        "keep all three papers but only the permitted sentence"
    );
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    let disk: String = connection
        .query_row("SELECT group_concat(item) FROM pool_item", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(
        !disk.contains("noncommercial body witness"),
        "pools must never persist noncommercial body text"
    );
    assert!(
        !disk.contains("unknown body witness"),
        "pools must never persist unknown-license body text"
    );
    let raw: Vec<u8> = connection
        .query_row(
            "SELECT body FROM reply WHERE source='litsense'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let raw = String::from_utf8(raw).unwrap();
    assert!(
        raw.contains("permitted body witness"),
        "the source cache may retain permitted text"
    );
    assert!(
        !raw.contains("noncommercial body witness"),
        "raw cache admission must filter noncommercial text"
    );
    assert!(
        !raw.contains("unknown body witness"),
        "raw cache admission must filter unknown-license body text"
    );
    assert!(
        !raw.contains("unlicensed ignored source field"),
        "unused source fields must not bypass text admission"
    );
    assert_withheld_fulltext(&fetch);
}
fn assert_withheld_fulltext(fetch: &Fetch) {
    let query = biomcp_types::ArticleText::part(
        biomcp_types::ArticleInput::Pmcid("PMC1002".parse().unwrap()),
        1,
    )
    .unwrap();
    let fulltext = biomcp_query::run(
        &biomcp_query::Ctx {
            fetch,
            budget: Duration::from_secs(20),
            fresh: false,
            ncbi_email: None,
            drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
        },
        &Request::ArticleText(query),
    );
    assert_eq!(
        fulltext.status(),
        biomcp_types::Status::Complete,
        "identity and refusal are a complete response: {fulltext:?}"
    );
    let ReplyData::ArticleTextResult {
        body: Some(body), ..
    } = fulltext.body
    else {
        panic!("fulltext identity response")
    };
    assert_eq!(
        body.pmcid.unwrap().to_string(),
        "PMC1002",
        "a withheld body retains its paper identity"
    );
    assert_eq!(
        body.text_withheld.unwrap().reason,
        biomcp_types::Reason::Disabled,
        "the reader must explain why text was withheld"
    );
}
#[test]
fn restricted_continuation_keeps_legacy_pool_rank_and_removes_disallowed_text() {
    let input = tempfile::tempdir().unwrap();
    record(input.path());
    let fetch = replay(input.path());
    let unrestricted = run(&fetch);
    let pool = unrestricted.page.unwrap().pool.unwrap();
    let fetch = fetch
        .restrict_uses([Use::PerRecord].into())
        .with_hosted_secrets(biomcp_fetch::Secrets::default());
    let query = serde_json::from_str::<ArticleSearch>(
        r#"{"text":"rights witness","exact_term":true,"source":["litsense"],"limit":3}"#,
    )
    .unwrap()
    .page(0, Some(pool.id.clone()));
    let restricted = biomcp_query::run(
        &biomcp_query::Ctx {
            fetch: &fetch,
            budget: Duration::from_secs(20),
            fresh: false,
            ncbi_email: None,
            drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
        },
        &Request::SearchArticle(query),
    );
    assert_eq!(
        restricted.page.unwrap().pool.unwrap(),
        pool,
        "rights checks must retain the original pool identity"
    );
    let ReplyData::ArticlePage {
        body: Some(page), ..
    } = restricted.body
    else {
        panic!("legacy article page")
    };
    assert_eq!(
        page.items.len(),
        3,
        "a restricted continuation must retain all original papers"
    );
    assert_eq!(
        page.items
            .iter()
            .filter(|row| row.sentence.is_some())
            .count(),
        1,
        "a retained pool must repeat the current per-record rule"
    );
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    let rows: String = connection
        .query_row("SELECT group_concat(item) FROM pool_item", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(
        !rows.contains("noncommercial body witness"),
        "rewritten pool rows must withhold noncommercial text"
    );
    assert!(
        !rows.contains("unknown body witness"),
        "rewritten pool rows must withhold unknown-license body text"
    );
}

#[path = "article_rights/contexts.rs"]
mod contexts;
#[path = "article_rights/lookups.rs"]
mod lookups;
#[path = "article_rights/routes.rs"]
mod routes;

#[path = "article_rights/authors.rs"]
mod authors;
