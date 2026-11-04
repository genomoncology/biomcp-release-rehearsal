//! Public checked query boundaries used by CLI and MCP.
use biodata::Text;
use biomcp_types::GeneInput;
use biomcp_types::Paged;
use biomcp_types::{GeneGet, GeneSearch};
use serde_json::json;
#[test]
fn get_rejects_empty_and_oversized_batches_at_constructor_and_json_boundary() {
    for count in [0, 1, 20, 21] {
        let inputs = vec!["BRAF".parse::<GeneInput>().unwrap(); count];
        assert_eq!(
            GeneGet::new(inputs.clone().into()).is_ok(),
            (1..=20).contains(&count)
        );
        assert_eq!(
            serde_json::from_value::<GeneGet>(json!({"input":inputs})).is_ok(),
            (1..=20).contains(&count)
        );
    }
    for sections in [json!([]), json!(["constraint", "all", "constraint"])] {
        let query: GeneGet =
            serde_json::from_value(json!({"input":["BRAF"],"section":sections})).unwrap();
        assert_eq!(query.sections().len(), sections.as_array().unwrap().len());
    }
    for sections in [
        json!(["Constraint"]),
        json!(["unknown"]),
        json!(null),
        json!("constraint"),
    ] {
        assert!(
            serde_json::from_value::<GeneGet>(json!({"input":["BRAF"],"section":sections}))
                .is_err()
        );
    }
}
#[test]
fn search_checks_selectors_and_page_bounds_at_both_entry_points() {
    for limit in [0, 1, 10, 50, 51] {
        assert_eq!(
            GeneSearch::new(Some(Text::new("BRAF").unwrap()), None, limit, u32::MAX).is_ok(),
            (1..=50).contains(&limit)
        );
        assert_eq!(
            serde_json::from_value::<GeneSearch>(json!({"query":"BRAF", "limit":limit})).is_ok(),
            (1..=50).contains(&limit)
        );
    }
    assert!(GeneSearch::new(None, None, 10, 0).is_err());
    assert!(serde_json::from_value::<GeneSearch>(json!({})).is_err());
    let query: GeneSearch = serde_json::from_value(json!({"band":"9p21.3"})).unwrap();
    assert_eq!(query.limit(), 10);
    assert_eq!(query.offset(), 0);
    assert_eq!(query.at_offset(50).offset(), 50);
}

/// Checked query transformations are a public API contract shared by CLI and MCP.
#[test]
fn term_drops_preserve_every_other_selector_and_restart_the_pool() {
    let original = json!({"text":"words", "gene":"NF2", "variant":"K322",
        "disease":"meningioma", "drug":"drug words", "author":"author words",
        "journal":"journal words", "since":"2000", "until":"2020", "type":"review",
        "source":["pubmed","europepmc"], "sort":"date", "open_access":true,
        "no_preprints":true, "retracted":"exclude", "offset":50,"pool":"old", "limit":20});
    let query: biomcp_types::ArticleSearch = serde_json::from_value(original.clone()).unwrap();
    let drops = query.term_drops();
    assert_eq!(drops.len(), 4);
    for (next, removed) in drops.iter().zip(["text", "variant", "disease", "drug"]) {
        let mut expected = original.clone();
        let fields = expected.as_object_mut().unwrap();
        fields.remove(removed);
        fields.remove("pool");
        fields.insert("offset".into(), json!(0));
        assert_eq!(serde_json::to_value(next).unwrap(), expected);
    }
    for request in [
        json!({"gene":"NF2"}),
        json!({"text":"two words", "gene":"NF2"}),
    ] {
        let query: biomcp_types::ArticleSearch = serde_json::from_value(request).unwrap();
        assert!(query.term_drops().is_empty());
    }
}
