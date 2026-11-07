use super::{hosted, record};
use std::time::Duration;

#[test]
fn citation_context_rights_preserve_papers_and_explain_withheld_text() {
    let input = tempfile::tempdir().unwrap();
    record(input.path());
    let empty = tempfile::tempdir().unwrap();
    biomcp_test_support::recordings::copy_capture(input.path(), empty.path(), |source, capture| {
        if source == "semantic-scholar" {
            for exchange in capture["exchanges"].as_array_mut().unwrap() {
                for row in exchange["body"]["value"]["data"].as_array_mut().unwrap() {
                    row["contexts"] = serde_json::json!([]);
                }
            }
        }
    })
    .unwrap();
    let empty_fetch = hosted(empty.path());
    let empty_leg = empty_fetch.leg(
        &biomcp_adapters::semantic_scholar::SPEC,
        empty_fetch.deadline(Duration::from_secs(20)),
    );
    let empty_result = biomcp_adapters::semantic_scholar::edges(
        &empty_leg,
        &biomcp_types::ArticleInput::Pmid("1001".parse().unwrap()),
        false,
        None,
    )
    .unwrap();
    assert_eq!(empty_result.data.len(), 3);
    assert!(
        empty_result
            .data
            .iter()
            .all(|edge| edge.text_withheld.is_none()),
        "no source context text must remain distinct from a policy refusal"
    );
    let fetch = hosted(input.path());
    let leg = fetch.leg(
        &biomcp_adapters::semantic_scholar::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let result = biomcp_adapters::semantic_scholar::edges(
        &leg,
        &biomcp_types::ArticleInput::Pmid("1001".parse().unwrap()),
        false,
        None,
    )
    .unwrap();
    let mut problems = Vec::new();
    let items: Vec<_> = result
        .data
        .into_iter()
        .map(|edge| edge.item(false, &mut problems).unwrap())
        .collect();
    assert_eq!(
        items.len(),
        3,
        "context admission must preserve every citing paper"
    );
    assert_eq!(
        items.iter().filter(|item| item.contexts.is_some()).count(),
        1,
        "only commercially permitted citation text may return"
    );
    assert_eq!(
        items
            .iter()
            .filter(|item| item
                .text_withheld
                .as_ref()
                .is_some_and(|notice| notice.url.is_some()))
            .count(),
        2,
        "withheld contexts must explain the refusal and give a reading destination"
    );
    let cached = biomcp_adapters::semantic_scholar::edges(
        &leg,
        &biomcp_types::ArticleInput::Pmid("1001".parse().unwrap()),
        false,
        None,
    )
    .unwrap();
    assert_eq!(
        cached
            .data
            .iter()
            .filter(|edge| edge.text_withheld.is_some())
            .count(),
        2,
        "reusing a mixed-license cache entry must retain prior refusal reasons"
    );
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    let body: Vec<u8> = connection
        .query_row(
            "SELECT body FROM reply WHERE source='semantic-scholar'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let body = String::from_utf8(body).unwrap();
    assert!(
        body.contains("permitted citation context"),
        "permitted citation text remains reusable"
    );
    assert!(
        !body.contains("noncommercial citation context"),
        "noncommercial contexts must never reach disk"
    );
    assert!(
        !body.contains("unknown malformed citation context"),
        "unused native fields must not grant their own citation-text license"
    );
    assert!(
        !body.contains("unknown citation context"),
        "unknown-license contexts must never reach disk"
    );
}
