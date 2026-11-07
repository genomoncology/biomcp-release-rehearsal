//! Unfinished license lookups are reported as such and never stored as the answer.
use super::{fixture, run};
use biomcp_fetch::Fetch;
use biomcp_types::{ArticleItem, ArticleSearch, Reason, ReplyData, Request, Status};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

fn items(out: biomcp_query::Outcome) -> Vec<ArticleItem> {
    let ReplyData::ArticlePage {
        body: Some(page), ..
    } = out.body
    else {
        panic!("article page: {out:?}")
    };
    page.items.into_vec()
}
fn stored(fetch: &Fetch, sql: &str) -> i64 {
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    connection.query_row(sql, [], |row| row.get(0)).unwrap()
}
const LITSENSE_REPLIES: &str = "SELECT count(*) FROM reply WHERE source='litsense'";

#[test]
fn failed_lookup_is_not_a_refusal_and_is_not_stored_as_whole() {
    let dir = tempfile::tempdir().unwrap();
    let (stub, fetch) = fixture::live(dir.path(), |request| {
        if request.contains("list-type") {
            return Ok("not a listing".into());
        }
        fixture::response(request)
    });
    let out = run(&fetch);
    let status = out.status();
    let notices = format!("{:?}", out.sources);
    let pool = out.page.clone().unwrap().pool.unwrap();
    let rows = items(out);
    assert_eq!(rows.len(), 3, "a failed lookup keeps every paper");
    assert!(
        rows.iter().all(|row| row.sentence.is_none()),
        "a failed lookup grants no body text"
    );
    assert!(
        rows.iter()
            .all(|row| row.text_withheld.as_ref().unwrap().reason == Reason::Unavailable),
        "a failed lookup must read differently from a refusal: {rows:?}"
    );
    assert_eq!(
        status,
        Status::Partial,
        "the reply must not claim a whole answer"
    );
    assert!(
        notices.contains("not stored"),
        "the source must say so: {notices}"
    );
    assert_eq!(
        stored(&fetch, LITSENSE_REPLIES),
        0,
        "a reply filtered on a failed lookup must not reach the source cache"
    );
    let query = serde_json::from_str::<ArticleSearch>(
        r#"{"text":"rights witness","exact_term":true,"source":["litsense"],"limit":3}"#,
    )
    .unwrap()
    .page(0, Some(pool.id));
    let again = biomcp_query::run(
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
        again.status(),
        Status::Partial,
        "a stored pool must keep saying it is not whole"
    );
    drop(stub);
}

#[test]
fn failed_lookup_pool_does_not_answer_a_new_search() {
    let dir = tempfile::tempdir().unwrap();
    let recovered = Arc::new(Mutex::new(false));
    let lookup = recovered.clone();
    let (stub, fetch) = fixture::live(dir.path(), move |request| {
        if request.contains("list-type") && !*lookup.lock().unwrap() {
            return Ok("not a listing".into());
        }
        fixture::response(request)
    });
    let first = run(&fetch);
    assert_eq!(first.status(), Status::Partial);
    let pool = first.page.unwrap().pool.unwrap().id;
    *recovered.lock().unwrap() = true;
    let second = run(&fetch);
    assert_eq!(
        second.status(),
        Status::Complete,
        "a new search must reach the recovered lookup"
    );
    assert_ne!(second.page.unwrap().pool.unwrap().id, pool);
    drop(stub);
}

#[test]
fn papers_past_the_lookup_cap_say_they_went_unchecked() {
    let papers: Vec<u32> = (2001..=2025).collect();
    let dir = tempfile::tempdir().unwrap();
    let (stub, fetch) = fixture::live(dir.path(), move |request| {
        Ok(fixture::many_papers(request, &papers))
    });
    let out = run(&fetch);
    let pool = out.page.clone().unwrap().pool.unwrap();
    assert_eq!(out.status(), Status::Partial, "capped pools are not whole");
    assert_eq!(pool.size, 25, "the cap must not remove papers");
    assert!(
        items(out).iter().all(|row| row.sentence.is_some()),
        "checked papers keep permitted text"
    );
    let query = serde_json::from_str::<ArticleSearch>(
        r#"{"text":"rights witness","exact_term":true,"source":["litsense"],"limit":3}"#,
    )
    .unwrap()
    .page(22, Some(pool.id));
    let late = biomcp_query::run(
        &biomcp_query::Ctx {
            fetch: &fetch,
            budget: Duration::from_secs(20),
            fresh: false,
            ncbi_email: None,
            drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
        },
        &Request::SearchArticle(query),
    );
    let rows = items(late);
    assert_eq!(rows.len(), 3, "late pages keep their papers");
    assert!(
        rows.iter().all(|row| row.sentence.is_none()
            && row.text_withheld.as_ref().unwrap().reason == Reason::Unavailable),
        "papers past the cap went unchecked, not refused: {rows:?}"
    );
    assert_eq!(
        stored(&fetch, LITSENSE_REPLIES),
        0,
        "capped replies are not stored"
    );
    drop(stub);
}

#[test]
fn pubmed_checks_licenses_before_its_own_call_and_keeps_an_unchecked_abstract() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let seen = log.clone();
    let dir = tempfile::tempdir().unwrap();
    let (stub, fetch) = fixture::live(dir.path(), move |request| {
        let name = if request.contains("esummary") {
            "esummary"
        } else {
            "efetch"
        };
        seen.lock().unwrap().push(name);
        if name == "esummary" {
            return Ok("not a summary".into());
        }
        fixture::response(request)
    });
    let leg = fetch.leg(
        &biomcp_adapters::pubmed::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let (articles, _) =
        biomcp_adapters::pubmed::fetch_articles(&leg, &["1002".parse().unwrap()], None).unwrap();
    let requests = log.lock().unwrap().clone();
    assert_eq!(
        requests.first(),
        Some(&"esummary"),
        "the lookup must finish before efetch takes the shared permit: {requests:?}"
    );
    assert!(
        requests.contains(&"efetch"),
        "efetch still runs: {requests:?}"
    );
    let article = &articles.first().unwrap().article;
    assert_eq!(
        article.abstract_parts.len(),
        1,
        "a failed lookup keeps the abstract under the source's terms"
    );
    assert_eq!(
        article.text_withheld.as_ref().unwrap().reason,
        Reason::Unavailable,
        "the reply must state what went unchecked"
    );
    assert_eq!(
        stored(&fetch, "SELECT count(*) FROM reply WHERE source='pubmed'"),
        0,
        "an unchecked abstract must not reach the source cache"
    );
    drop(stub);
}

#[test]
fn repeat_reads_keep_refusal_markers_and_write_nothing_new() {
    let dir = tempfile::tempdir().unwrap();
    let (stub, fetch) = fixture::live(dir.path(), fixture::response);
    let read_pubmed = || {
        let leg = fetch.leg(
            &biomcp_adapters::pubmed::SPEC,
            fetch.deadline(Duration::from_secs(20)),
        );
        let (articles, _) =
            biomcp_adapters::pubmed::fetch_articles(&leg, &["1002".parse().unwrap()], None)
                .unwrap();
        assert!(articles.first().unwrap().article.text_withheld.is_some());
    };
    read_pubmed();
    read_pubmed();
    read_pubmed();
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    let body: String = connection
        .query_row(
            "SELECT CAST(body AS TEXT) FROM reply WHERE CAST(body AS TEXT) LIKE '%PubmedArticleSet%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        body.matches("<BioMcpTextWithheld/>").count(),
        1,
        "cached reads must not repeat the PubMed marker"
    );

    let first = run(&fetch);
    let pool = first.page.clone().unwrap().pool.unwrap();
    // Any item rewrite on an unchanged page now fails the read.
    connection
        .execute_batch(
            "CREATE TRIGGER no_rewrite BEFORE UPDATE OF item ON pool_item \
             BEGIN SELECT RAISE(ABORT, 'unchanged row rewritten'); END",
        )
        .unwrap();
    let query = serde_json::from_str::<ArticleSearch>(
        r#"{"text":"rights witness","exact_term":true,"source":["litsense"],"limit":3}"#,
    )
    .unwrap()
    .page(0, Some(pool.id));
    let again = biomcp_query::run(
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
        items(again).len(),
        3,
        "an unchanged page reads without a write"
    );

    let marked = br#"[{"pmid":1001,"biomcp_text_withheld":true}]"#.as_slice();
    connection
        .execute("UPDATE reply SET body=?1 WHERE source='litsense'", [marked])
        .unwrap();
    let leg = fetch.leg(
        &biomcp_adapters::litsense::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let rows = biomcp_adapters::litsense::search(&leg, "rights witness")
        .unwrap()
        .value;
    assert_eq!(rows.len(), 1, "the cached row keeps its paper");
    let row = rows.first().unwrap();
    assert!(row.sentence.is_none(), "a cached refusal has no text");
    assert_eq!(
        row.text_withheld.as_ref().unwrap().reason,
        Reason::Disabled,
        "a later commercial lookup must not erase the cached refusal"
    );
    drop(stub);
}

#[test]
fn a_stated_restriction_withholds_the_abstract_in_any_spelling() {
    for license in [
        "CC BY-NC 4.0",
        "cc by-nc-nd/4.0",
        "https://creativecommons.org/licenses/by-nc-sa/4.0/",
        "CCBYNC",
        "Non-Commercial",
        "TDM only",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let (stub, fetch) = fixture::live(dir.path(), move |_| {
            Ok(serde_json::json!({"result": {
                "id": "PPR1002", "source": "PPR", "title": "Preprint identity",
                "license": license, "abstractText": "spelled license witness"
            }})
            .to_string())
        });
        let leg = fetch.leg(
            &biomcp_adapters::europepmc::SPEC,
            fetch.deadline(Duration::from_secs(20)),
        );
        let (article, _) =
            biomcp_adapters::europepmc::core(&leg, &"PPR1002".parse().unwrap()).unwrap();
        assert!(
            article.value.abstract_parts.is_empty() && article.value.text_withheld.is_some(),
            "{license} states a restriction"
        );
        drop(stub);
    }
}

fn both_sources(fetch: &Fetch) -> biomcp_query::Outcome {
    biomcp_query::run(
        &biomcp_query::Ctx {
            fetch,
            budget: Duration::from_secs(20),
            fresh: false,
            ncbi_email: None,
        },
        &Request::SearchArticle(
            serde_json::from_str::<ArticleSearch>(
                r#"{"text":"rights witness","source":["pubmed","litsense"],"limit":3}"#,
            )
            .unwrap(),
        ),
    )
}

#[test]
fn unchecked_text_reaches_the_reply_and_not_the_store_and_a_repeat_rechecks() {
    let healthy = Arc::new(AtomicBool::new(false));
    let dir = tempfile::tempdir().unwrap();
    let state = healthy.clone();
    let (stub, fetch) = fixture::live(dir.path(), move |request| {
        if request.contains("sentences") {
            return Ok(serde_json::json!([
                {"pmid":2001,"section":"ABSTRACT","text":"unchecked abstract witness"},
                {"pmid":2002,"section":"RESULTS","text":"unchecked body witness"}
            ])
            .to_string());
        }
        if request.contains("esearch") {
            return Ok(serde_json::json!({"esearchresult":{
                "count":"2","retmax":"2","retstart":"0","idlist":["2001","2002"]
            }})
            .to_string());
        }
        if !state.load(Ordering::SeqCst) && request.contains("list-type") {
            return Ok("not a listing".into());
        }
        Ok(fixture::many_papers(request, &[2001, 2002]))
    });
    let out = both_sources(&fetch);
    assert_eq!(out.status(), Status::Partial, "this pool is not whole");
    let rows = items(out);
    assert_eq!(rows.len(), 2, "a failed lookup keeps every paper");
    assert!(
        rows[0].sentence.is_some() && rows[1].sentence.is_none(),
        "the abstract stays in the reply and the body does not: {rows:?}"
    );
    assert!(
        rows.iter()
            .all(|row| row.text_withheld.as_ref().unwrap().reason == Reason::Unavailable),
        "a paper found by two sources keeps its reason: {rows:?}"
    );
    assert_eq!(
        stored(
            &fetch,
            "SELECT count(*) FROM pool_item WHERE item LIKE '%unchecked%witness%'"
        ),
        0,
        "text from an unchecked paper must not reach a stored row"
    );
    healthy.store(true, Ordering::SeqCst);
    let rows = items(both_sources(&fetch));
    assert!(
        rows.iter()
            .all(|row| row.sentence.is_some() && row.text_withheld.is_none()),
        "a repeat search rechecks what the failed lookup left out: {rows:?}"
    );
    drop(stub);
}
