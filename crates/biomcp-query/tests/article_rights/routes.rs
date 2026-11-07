use super::{hosted, record, run};
use biomcp_fetch::Fetch;
use biomcp_types::ReplyData;
use std::time::Duration;

#[test]
fn restricted_abstracts_and_passages_follow_the_same_record_policy_before_cache_writes() {
    let input = tempfile::tempdir().unwrap();
    record(input.path());
    let fetch = hosted(input.path());
    check_pubmed(&fetch);
    check_pubtator(&fetch);
    check_europepmc(&fetch);
    check_abstract_sentences(&fetch);
    check_cache(&fetch);
}
fn check_pubmed(fetch: &Fetch) {
    let leg = fetch.leg(
        &biomcp_adapters::pubmed::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let (articles, _) =
        biomcp_adapters::pubmed::fetch_articles(&leg, &["1002".parse().unwrap()], None).unwrap();
    assert_eq!(
        articles.len(),
        1,
        "withholding an abstract must preserve the paper"
    );
    assert!(
        articles.first().unwrap().article.abstract_parts.is_empty(),
        "an explicit noncommercial abstract must be withheld"
    );
    assert!(
        articles.first().unwrap().article.text_withheld.is_some(),
        "the paper must explain its missing abstract"
    );
}
fn check_pubtator(fetch: &Fetch) {
    let leg = fetch.leg(
        &biomcp_adapters::pubtator::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let annotations = biomcp_adapters::pubtator::annotations(
        &leg,
        &[
            "1001".parse().unwrap(),
            "1002".parse().unwrap(),
            "1003".parse().unwrap(),
        ],
    )
    .unwrap();
    assert_eq!(
        annotations
            .iter()
            .map(|row| row.id.to_string())
            .collect::<Vec<_>>(),
        ["1001", "1002", "1003"],
        "annotations must preserve every requested paper"
    );
}
fn check_europepmc(fetch: &Fetch) {
    let leg = fetch.leg(
        &biomcp_adapters::europepmc::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let (noncommercial, _) =
        biomcp_adapters::europepmc::core(&leg, &"PPR1002".parse().unwrap()).unwrap();
    let (unknown, _) = biomcp_adapters::europepmc::core(&leg, &"PPR1003".parse().unwrap()).unwrap();
    let (tdm, _) = biomcp_adapters::europepmc::core(&leg, &"PPR1004".parse().unwrap()).unwrap();
    assert!(
        tdm.value.abstract_parts.is_empty(),
        "a text-mining license withholds the abstract"
    );
    assert!(
        noncommercial.value.abstract_parts.is_empty(),
        "Europe PMC's explicit noncommercial license must withhold its abstract"
    );
    assert!(
        noncommercial.value.text_withheld.is_some(),
        "Europe PMC must explain the withheld abstract"
    );
    assert_eq!(
        unknown.value.abstract_parts.len(),
        1,
        "unknown abstract licenses retain the source's terms"
    );
}
fn check_cache(fetch: &Fetch) {
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    let stored: String = connection
        .query_row(
            "SELECT group_concat(CAST(body AS TEXT)) FROM reply",
            [],
            |row| row.get(0),
        )
        .unwrap();
    for forbidden in [
        "noncommercial abstract witness",
        "noncommercial passage witness",
        "unknown passage witness",
        "PPR1002 abstract witness",
        "PPR1004 abstract witness",
        "noncommercial title sentence",
    ] {
        assert!(
            !stored.contains(forbidden),
            "restricted cache retained {forbidden}"
        );
    }
    assert!(
        stored.contains("permitted passage witness"),
        "permitted body text may remain cached"
    );
    assert!(
        stored.contains("unknown abstract passage retained"),
        "unknown abstract licenses follow source terms"
    );
    assert!(
        stored.contains("PPR1003 abstract witness"),
        "unknown Europe PMC abstract licenses follow source terms"
    );
}

fn check_abstract_sentences(fetch: &Fetch) {
    let leg = fetch.leg(
        &biomcp_adapters::litsense::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let result = biomcp_adapters::litsense::search(&leg, "abstract rights witness").unwrap();
    assert_eq!(
        result.value.len(),
        3,
        "abstract sentence filtering must preserve every paper"
    );
    assert_eq!(
        result
            .value
            .iter()
            .filter(|row| row.sentence.is_some())
            .count(),
        2,
        "unknown abstract licenses follow source terms, while explicit NC title text is withheld"
    );
}

#[test]
fn exhausted_license_budget_returns_papers_without_text_or_license_requests() {
    let original = tempfile::tempdir().unwrap();
    record(original.path());
    let delayed = tempfile::tempdir().unwrap();
    biomcp_test_support::recordings::copy_capture(
        original.path(),
        delayed.path(),
        |source, capture| {
            if source == "litsense" {
                for exchange in capture["exchanges"].as_array_mut().unwrap() {
                    exchange["delay_ms"] = 14_950.into();
                }
            }
        },
    )
    .unwrap();
    let fetch = hosted(delayed.path());
    let out = run(&fetch);
    assert_eq!(
        out.status(),
        biomcp_types::Status::Partial,
        "a timed-out lookup is not a whole answer"
    );
    let ReplyData::ArticlePage {
        body: Some(page), ..
    } = out.body
    else {
        panic!("delayed article page: {out:?}")
    };
    assert_eq!(
        page.items.len(),
        3,
        "license timeout must preserve every paper"
    );
    assert!(
        page.items.iter().all(|row| row.sentence.is_none()),
        "insufficient license budget must withhold body sentences"
    );
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    let lookups: i64 = connection
        .query_row(
            "SELECT count(*) FROM reply WHERE source='pmc-oa'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        lookups, 0,
        "the remaining source budget must reserve time for paper identities"
    );
}
