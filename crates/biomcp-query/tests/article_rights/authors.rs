use super::{hosted, record, replay};
use biomcp_fetch::Fetch;
use biomcp_types::{AuthorPapersQuery, ReplyData, Request, SourceNotice, Use};
use std::time::Duration;

pub(super) fn query() -> AuthorPapersQuery {
    serde_json::from_str(r#"{"id":"semantic-scholar:123","section":"details","limit":3}"#).unwrap()
}
fn run(fetch: &Fetch, query: AuthorPapersQuery) -> biomcp_query::Outcome {
    biomcp_query::run(
        &biomcp_query::Ctx {
            fetch,
            budget: Duration::from_secs(20),
            fresh: false,
            ncbi_email: None,
        },
        &Request::AuthorPapers(query),
    )
}
fn assert_refusal(out: &biomcp_query::Outcome) {
    let ReplyData::AuthorPaperDetails {
        body: Some(details),
        ..
    } = &out.body
    else {
        panic!("author details: {out:?}")
    };
    assert_eq!(
        details.items.len(),
        3,
        "abstract policy must keep every paper"
    );
    assert_eq!(
        details
            .items
            .iter()
            .filter(|row| row.abstract_text.is_some())
            .count(),
        2,
        "only the explicit NC abstract must be withheld"
    );
    let notice = out.sources.iter().flat_map(|source| &source.notices).any(|notice| {
        matches!(notice, SourceNotice::RequestGuidance { message, .. }
            if message.contains("withheld") && message.contains("https://pubmed.ncbi.nlm.nih.gov/1002/"))
    });
    assert!(
        notice,
        "the withheld abstract needs a reason and reading link: {out:?}"
    );
}
#[test]
fn author_abstract_diagnostics_survive_source_cache_and_continuation() {
    let input = tempfile::tempdir().unwrap();
    record(input.path());
    let fetch = hosted(input.path());
    let first = run(&fetch, query());
    assert_refusal(&first);
    let pool = first.page.unwrap().pool.unwrap();
    let cx = fetch.cache_access().unwrap().connect().unwrap();
    cx.execute("DELETE FROM pool_current WHERE pool=?1", [pool.id.as_str()])
        .unwrap();
    assert_refusal(&run(&fetch, query()));
    assert_refusal(&run(&fetch, query().with_pool(pool.id)));
}
#[test]
fn legacy_author_continuation_marks_the_abstract_removed_during_recheck() {
    let input = tempfile::tempdir().unwrap();
    record(input.path());
    let fetch = replay(input.path());
    let first = run(&fetch, query());
    let pool = first.page.unwrap().pool.unwrap();
    let cx = fetch.cache_access().unwrap().connect().unwrap();
    cx.execute("DELETE FROM reply WHERE source='semantic-scholar'", [])
        .unwrap();
    let fetch = fetch
        .restrict_uses([Use::PerRecord].into())
        .with_hosted_secrets(biomcp_fetch::Secrets::default());
    assert_refusal(&run(&fetch, query().with_pool(pool.id)));
    let payload: String = cx
        .query_row("SELECT payload FROM trial_snapshot LIMIT 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(
        !payload.contains("author abstract 1002"),
        "rechecked snapshots must not persist the withheld abstract"
    );
}
