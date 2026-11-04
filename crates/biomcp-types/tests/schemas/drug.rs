//! Public drug header and card disclosure boundaries.
use biomcp_types::{DrugCard, DrugHeader};
use serde_json::json;

#[test]
#[cfg(feature = "schema")]
fn header_and_card_exclude_deeper_disclosures_from_the_public_contract() {
    let header = json!({"id":"CHEMBL941", "source":"chembl", "name":"native name", "identity_level":"unknown"});
    let decoded: DrugHeader = serde_json::from_value(header.clone()).expect("header");
    assert_eq!(serde_json::to_value(decoded).expect("header JSON"), header);
    for extra in [
        "approval",
        "approval_date",
        "authorization_events",
        "label_text",
        "interactions",
        "targets",
    ] {
        let mut value = header.clone();
        value[extra] = json!([]);
        let decoded = serde_json::from_value::<DrugHeader>(value).expect("open header");
        assert_eq!(serde_json::to_value(decoded).expect("header JSON"), header);
        let value = json!({"header":header, extra:[]});
        let decoded = serde_json::from_value::<DrugCard>(value).expect("open card");
        assert_eq!(
            serde_json::to_value(decoded).expect("card JSON"),
            json!({"header":header})
        );
    }
    let schema = serde_json::to_value(schemars::schema_for!(DrugHeader)).expect("header schema");
    let keys: Vec<_> = schema["properties"]
        .as_object()
        .expect("header properties")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "id",
            "identity_level",
            "matched_name",
            "name",
            "name_resolution",
            "source",
            "type",
            "url",
            "version"
        ]
    );
    let schema = serde_json::to_value(schemars::schema_for!(DrugCard)).expect("card schema");
    let text = serde_json::to_string(&schema).expect("schema JSON");
    for forbidden in [
        "AuthorizationEvent",
        "DrugLabelSection",
        "DrugInteractionClaim",
        "DrugTargetRelation",
    ] {
        assert!(!text.contains(forbidden), "{forbidden}");
    }
}
