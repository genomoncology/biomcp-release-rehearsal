//! Feature diagnostic-types: checked source-owned records before adapters exist.
use biodata::*;
use biomcp_types::Paged;
use biomcp_types::*;
use serde_json::{Value, json};

#[expect(
    clippy::unwrap_used,
    reason = "Contract setup must fail when synthetic public values cannot round-trip"
)]
fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(value: Value) -> Value {
    serde_json::to_value(serde_json::from_value::<T>(value).unwrap()).unwrap()
}

#[test]
fn native_id_grammars_refuse_wrappers_and_incomplete_identities() {
    for value in ["GTR000006692.3", "GTR000511179.13"] {
        assert_eq!(roundtrip::<GtrTestId>(json!(value)), value);
    }
    for value in [
        "GTR000006692",
        "gtr:GTR000006692.3",
        "GTR6692.3",
        "GTR000006692.0",
        "GTR000006692.03",
        "GTR000006692.+3",
    ] {
        assert!(serde_json::from_value::<GtrTestId>(json!(value)).is_err());
    }
    assert_eq!(
        roundtrip::<WhoProductId>(json!("0466-046-00")),
        "0466-046-00"
    );
    for id in ["13641-013-00", "10304-118-00", "10307-04545-00"] {
        assert_eq!(roundtrip::<WhoProductId>(json!(id)), id);
    }
    assert_eq!(roundtrip::<PmaNumber>(json!("P150013")), "P150013");
    assert_eq!(roundtrip::<PmaSupplement>(json!("S020")), "S020");
    assert_eq!(
        roundtrip::<FdaDrugApplication>(json!("NDA021335")),
        "NDA021335"
    );
    for value in ["021335", "NDA21335", "drugsfda:NDA021335", "nda021335"] {
        assert!(serde_json::from_value::<FdaDrugApplication>(json!(value)).is_err());
    }
    for value in ["P150013", "P150013/S020", "K173492", "DEN170019"] {
        assert_eq!(roundtrip::<DiagnosticDeviceId>(json!(value)), value);
    }
    for value in [
        "fda:P150013",
        "P150013S020",
        "P150013/S000",
        "DEN17",
        "K173492/S001",
    ] {
        assert!(serde_json::from_value::<DiagnosticDeviceId>(json!(value)).is_err());
    }
}

#[test]
fn checked_search_rejects_cross_source_selectors_before_io() {
    for query in [
        json!({"source":"gtr","gene_id":"672"}),
        json!({"source":"gtr","gene":"BRCA1","type":"Clinical"}),
        json!({"source":"gtr","condition_id":"MedGen:C2676676"}),
        json!({"source":"fda-cdx","device_id":"P150013/S020"}),
        json!({"source":"fda-cdx","drug_application":"BLA125514"}),
        json!({"source":"fda-cdx","biomarker":"KRAS"}),
    ] {
        let checked: DiagnosticSearch = serde_json::from_value(query.clone()).unwrap();
        let mut expected = roundtrip::<DiagnosticSearch>(query);
        assert_eq!(expected["limit"], 5);
        expected["offset"] = json!(5);
        assert_eq!(
            serde_json::to_value(checked.at_offset(5)).unwrap(),
            expected
        );
    }
    for query in [
        json!({"gene":"BRCA1"}),
        json!({"source":"all"}),
        json!({"source":"gtr","gene":"BRCA1","gene_id":"672"}),
        json!({"source":"gtr","condition_id":"MONDO:0007254"}),
        json!({"source":"gtr","condition_id":"MedGen:2676676"}),
        json!({"source":"gtr","gene_id":"672","condition_id":"MedGen:C2676676"}),
        json!({"source":"gtr","device_id":"P150013"}),
        json!({"source":"fda-cdx","gene_id":"672"}),
        json!({"source":"fda-cdx","condition_id":"MedGen:C2676676"}),
        json!({"source":"fda-cdx","type":"Clinical"}),
        json!({"source":"fda-cdx","device_id":"DEN170019"}),
        json!({"source":"gtr","limit":0}),
        json!({"source":"fda-cdx","drug_cell":"Keytruda"}),
    ] {
        assert!(
            serde_json::from_value::<DiagnosticSearch>(query.clone()).is_err(),
            "{query}"
        );
    }
}

#[test]
fn checked_get_keeps_base_supplement_and_section_scope() {
    for query in [
        json!({"source":"gtr","id":"GTR000006692.3","section":"genes"}),
        json!({"source":"gtr","id":"GTR000006692.3","section":"conditions"}),
        json!({"source":"gtr","id":"GTR000006692.3","section":"methods"}),
        json!({"source":"who-ivd","id":"0466-046-00"}),
        json!({"source":"fda-pma","id":"P150013","section":"supplements"}),
        json!({"source":"fda-pma","id":"P150013","supplement":"S020","section":"regulatory"}),
    ] {
        let get: DiagnosticGet = serde_json::from_value(query.clone()).unwrap();
        let mut expected = roundtrip::<DiagnosticGet>(query);
        expected["section_offset"] = json!(5);
        assert_eq!(
            serde_json::to_value(get.at_section_offset(5)).unwrap(),
            expected
        );
    }
    for query in [
        json!({"id":"GTR000006692.3"}),
        json!({"source":"gtr","id":"GTR000006692"}),
        json!({"source":"gtr","id":"GTR000006692.3","section":"all"}),
        json!({"source":"gtr","id":"GTR000006692.3","supplement":"S020"}),
        json!({"source":"who-ivd","id":"0466-046-00","section":"genes"}),
        json!({"source":"fda-pma","id":"P150013","section":"genes"}),
        json!({"source":"fda-pma","id":"P150013","supplement":"S020","section":"supplements"}),
        json!({"source":"fda-pma","id":"P150013/S020"}),
        json!({"source":"fda-pma","id":"K173492"}),
        json!({"source":"fda-cdx","id":"P150013"}),
        json!({"source":"gtr","id":"GTR000006692.3","full":true}),
    ] {
        assert!(
            serde_json::from_value::<DiagnosticGet>(query.clone()).is_err(),
            "{query}"
        );
    }
}

fn gene() -> Value {
    json!(
        {"test_id":"GTR000006692.3","test_type":"Clinical","gene":{"ncbi_gene_id":"672","label":"Synthetic symbol"},
        "native_medgen":"MedGen:C0376571","mim_number":"","object_name":"Synthetic native gene label"}
    )
}
fn condition() -> Value {
    json!(
        {"test_id":"GTR000006692.3","test_type":"Clinical","condition":{"id":"MedGen:C2676676","label":"Synthetic condition"},
        "mim_number":"604370","snomed_ct_id":""}
    )
}
fn method() -> Value {
    json!({"test_id":"GTR000006692.3","list":"methods","index":2,"value":"Synthetic unpaired method"})
}
fn pma_header() -> Value {
    json!(
        {"pma_number":"P150013","supplement_number":"","trade_name":"Synthetic device","applicant":"Synthetic applicant",
        "decision_date":"20200101","decision_code":"APPR","url":"https://example.invalid/pma/P150013"}
    )
}
fn who() -> Value {
    let rows: Vec<_> = (0..9)
        .map(|i| {
            json!(
                {"product_name":"Synthetic product","product_code":format!("code-{i}"),"assay_format":"Synthetic format",
                "manufacturer":"Synthetic manufacturer","marker":format!("native marker {i}"),"regulatory_version":"Uninterpreted source prose",
                "prequalification_year":"2024"}
            )
        })
        .collect();
    json!({"id":"0466-046-00","native_rows":rows})
}
fn companion(group: bool) -> Value {
    let headers = if group {
        vec![
            "Diagnostic Name\n(Manufacturer)",
            "Indication(s) - Sample Type",
            "PMA (Approval Date)",
            "Device Indication for a Specific Group of Oncology Therapeutic Products and Trade Name (Generic) – NDA/BLA",
        ]
    } else {
        vec![
            "Diagnostic Name (Manufacturer)",
            "Indication - Sample Type",
            "Drug Trade Name (Generic) NDA / BLA",
            "Biomarker(s)",
            "Biomarker(s) (Details)",
            "PMA / 510(k) /\n513(f)(2) / HDE (Approval / Clearance / Grant Date)",
        ]
    };
    let cells: serde_json::Map<_, _> = headers
        .into_iter()
        .map(|h| {
            (
                h.to_owned(),
                json!({"text":format!("Whole synthetic {h} cell"), "links":[{"href":"https://example.invalid/cell-owned"}]}),
            )
        })
        .collect();
    json!(
        {"kind":if group {"group"} else {"individual"},"snapshot":"synthetic-revision","index":7,"cells":cells,"device_ids":["P150013/S020",
        "K173492","DEN170019"],"drug_applications":[{"id":"NDA021335","url":"https://example.invalid/owned/NDA021335"},{"id":"BLA125514",
        "url":"https://example.invalid/owned/BLA125514"}]}
    )
}

#[test]
fn native_memberships_keep_namespace_and_cannot_become_statements() {
    for row in [gene(), condition(), method()] {
        assert!(serde_json::from_value::<Statement>(row).is_err());
    }
    assert_eq!(roundtrip::<GtrGeneRelation>(gene()), gene());
    assert_eq!(roundtrip::<GtrConditionRelation>(condition()), condition());
    assert_eq!(roundtrip::<GtrMethod>(method()), method());
    assert!(serde_json::from_value::<GtrGeneRelation>(condition()).is_err());
    assert!(serde_json::from_value::<GtrConditionRelation>(gene()).is_err());
    for kind in ["Clinical", "Research", "Clinical microbial"] {
        assert_eq!(roundtrip::<GtrTestType>(json!(kind)), kind);
    }
    assert!(serde_json::from_value::<GtrTestType>(json!("clinical")).is_err());
    for list in ["methods", "method_categories", "platforms"] {
        assert_eq!(roundtrip::<GtrMethodList>(json!(list)), list);
    }
    assert!(serde_json::from_value::<GtrMethodList>(json!("all")).is_err());
}

#[test]
fn native_test_rows_keep_empty_cells_and_all_five_status_flags() {
    let mut row = json!({"test_accession_ver":"GTR000551465.2"});
    for key in [
        "name_of_laboratory",
        "name_of_institution",
        "facility_state",
        "facility_postcode",
        "facility_country",
        "CLIA_number",
        "state_licenses",
        "state_license_numbers",
        "lab_test_id",
        "last_touch_date",
        "lab_test_name",
        "manufacturer_test_name",
        "test_development",
        "lab_id",
        "condition_identifiers",
        "indication_types",
        "inheritances",
        "method_categories",
        "methods",
        "platforms",
        "genes",
        "drug_responses",
        "now_current",
        "test_currStat",
        "test_pubStat",
        "lab_currStat",
        "lab_pubStat",
        "test_create_date",
        "test_deletion_date",
        "test_order_code",
    ] {
        row[key] = json!("");
    }
    for (key, value) in [
        ("now_current", "0"),
        ("test_currStat", "1"),
        ("test_pubStat", "3"),
        ("lab_currStat", "72"),
        ("lab_pubStat", "4"),
    ] {
        row[key] = json!(value);
    }
    let first = roundtrip::<GtrTestVersion>(row.clone());
    row["last_touch_date"] = json!("2026-06-29");
    let second = roundtrip::<GtrTestVersion>(row.clone());
    assert_ne!(first, second);
    assert_eq!(second["CLIA_number"], "");
    assert_eq!(second["lab_currStat"], "72");
    assert_eq!(second["test_pubStat"], "3");
    let records = [
        json!({"kind":"gtr_test_version","record":first}),
        json!({"kind":"gtr_test_version","record":second}),
    ];
    assert_eq!(
        roundtrip::<Vec<DiagnosticRecord>>(json!(records)),
        json!(records)
    );
}

#[test]
fn who_cards_preserve_every_complete_code_row_and_original_scope() {
    let value = who();
    let card = roundtrip::<WhoIvdProduct>(value.clone());
    assert_eq!(card, value);
    assert_eq!(card["native_rows"].as_array().unwrap().len(), 9);
    assert_eq!(card["native_rows"][8]["marker"], "native marker 8");
    assert!(serde_json::from_value::<Statement>(card).is_err());
}

#[test]
fn pma_absent_empty_and_explicit_supplement_roles_stay_distinct() {
    let absent = json!({"pma_number":"P150013"});
    assert_eq!(roundtrip::<DeviceSubmission>(absent.clone()), absent);
    for sub in ["", "S020"] {
        let row = json!(
            {"pma_number":"P150013","supplement_number":sub,"ao_statement":"Whole synthetic source statement","supplement_type":"",
            "supplement_reason":""}
        );
        assert_eq!(roundtrip::<DeviceSubmission>(row.clone()), row);
    }
    for sub in ["S000", "020", "S20", "base"] {
        assert!(
            serde_json::from_value::<DeviceSubmission>(
                json!({"pma_number":"P150013","supplement_number":sub})
            )
            .is_err()
        );
    }
    let header =
        json!({"pma_number":"P150013", "supplement_number":"", "url":"https://example.invalid"});
    let mut extended = header.clone();
    extended["ao_statement"] = json!("Whole statement must stay behind regulatory");
    assert_eq!(roundtrip::<DiagnosticPmaHeader>(extended), header);
}

#[test]
fn physical_companion_tables_keep_exact_headers_whole_cells_and_owned_ids() {
    for group in [false, true] {
        let row = companion(group);
        assert_eq!(roundtrip::<CompanionDiagnosticRelation>(row.clone()), row);
        assert!(serde_json::from_value::<Statement>(row.clone()).is_err());
        let mut invalid = row.clone();
        invalid["cells"]["invented column"] = json!({"text":"invented","links":[]});
        assert_eq!(roundtrip::<CompanionDiagnosticRelation>(invalid), row);
        let mut wrong_kind = row;
        wrong_kind["kind"] = json!(if group { "individual" } else { "group" });
        assert!(serde_json::from_value::<CompanionDiagnosticRelation>(wrong_kind).is_err());
    }
    let mut bad_id = companion(false);
    bad_id["drug_applications"][0]["id"] = json!("21335");
    assert!(serde_json::from_value::<CompanionDiagnosticRelation>(bad_id).is_err());
}

#[test]
fn conflict_candidates_have_no_selected_name_or_card() {
    let value = json!(
        {"identity_status":"conflict","id":"GTR000551465.2","problem":{"source":"pubmed","record":"GTR000551465.2",
        "field":"test_accession_ver","code":"value.unknown"}}
    );
    // This pre-registry synthetic source is not a GTR provider observation.
    assert_eq!(roundtrip::<DiagnosticCandidate>(value.clone()), value);
    assert!(serde_json::from_value::<DiagnosticCard>(value.clone()).is_err());
    let mut invalid = value.clone();
    invalid["name"] = json!("Silent winner");
    assert_eq!(roundtrip::<DiagnosticCandidate>(invalid), value);
}

#[test]
fn continuation_preserves_revision_filters_and_source_checked_sections() {
    let search: DiagnosticSearch = serde_json::from_value(
        json!({"source":"gtr","gene_id":"672","type":"Clinical","bundle_revision":"revision-1"}),
    )
    .unwrap();
    assert_eq!(search.source(), DiagnosticSearchSource::Gtr);
    assert_eq!(search.gene_id().unwrap().to_string(), "672");
    assert!(search.gene().is_none());
    assert!(search.condition_id().is_none());
    assert!(search.device_id().is_none());
    assert!(search.drug_application().is_none());
    assert!(search.biomarker().is_none());
    assert_eq!(search.test_type(), Some(GtrTestType::Clinical));
    let next = search
        .with_bundle_revision(Text::new("revision-2").unwrap())
        .at_offset(5);
    assert_eq!(next.bundle_revision().unwrap().as_str(), "revision-2");
    assert_eq!((next.limit(), next.offset()), (5, 5));
    let get: DiagnosticGet =
        serde_json::from_value(json!({"source":"fda-pma","id":"P150013","supplement":"S020"}))
            .unwrap();
    assert_eq!(get.id().as_str(), "P150013");
    assert_eq!(get.source(), DiagnosticGetSource::FdaPma);
    assert_eq!(get.supplement().unwrap().as_str(), "S020");
    assert!(get.with_section(DiagnosticSection::Supplements).is_err());
    assert!(get.with_section(DiagnosticSection::Genes).is_err());
    let next = get
        .with_section(DiagnosticSection::Regulatory)
        .unwrap()
        .with_bundle_revision(Text::new("revision-2").unwrap())
        .at_section_offset(2);
    assert_eq!(next.section(), Some(DiagnosticSection::Regulatory));
    assert_eq!(next.section_offset(), 2);
    assert_eq!(next.section_limit(), 1);
    assert_eq!(next.bundle_revision().unwrap().as_str(), "revision-2");
}

#[test]
fn regulatory_default_counts_one_fragment_and_explicit_limits_remain_explicit() {
    let base = json!({"source":"fda-pma","id":"P150013","section":"regulatory"});
    let get: DiagnosticGet = serde_json::from_value(base.clone()).unwrap();
    assert_eq!(get.section_limit(), 1);
    for limit in [json!(0), json!(null), json!(51)] {
        let mut bad = base.clone();
        bad["section_limit"] = limit;
        assert!(serde_json::from_value::<DiagnosticGet>(bad).is_err());
    }
    let mut explicit = base;
    explicit["section_limit"] = json!(5);
    let get: DiagnosticGet = serde_json::from_value(explicit).unwrap();
    assert_eq!(get.section_limit(), 5);
}

#[cfg(feature = "schema")]
#[expect(
    clippy::unwrap_used,
    reason = "Contract setup requires valid synthetic bodies, queries and generated schemas"
)]
fn envelope<T>(query: Value, body: Value)
where
    T: ResponseBody + schemars::JsonSchema + serde::de::DeserializeOwned,
    T::Query: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let body: T = serde_json::from_value(body).unwrap();
    let query = serde_json::from_value(query).unwrap();
    let value = serde_json::to_value(Envelope::new(Status::Complete, query, Some(body))).unwrap();
    let schema = serde_json::to_value(schemars::schema_for!(Envelope<T>)).unwrap();
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    assert!(validator.is_valid(&value), "{value}");
    assert_eq!(
        value.get("schema").and_then(Value::as_str),
        Some(T::SCHEMA),
        "Diagnostic replies must retain their versioned envelope contract"
    );
}

#[cfg(feature = "schema")]
#[test]
fn all_eight_consumer_schemas_accept_their_source_owned_views() {
    let search = json!({"source":"gtr","gene_id":"672"});
    let get = json!({"source":"gtr","id":"GTR000006692.3"});
    envelope::<DiagnosticPage>(search, json!({"items":[]}));
    let mut card = who();
    card["kind"] = json!("who-ivd-product");
    envelope::<DiagnosticCard>(json!({"source":"who-ivd","id":"0466-046-00"}), card);
    envelope::<DiagnosticGenes>(get.clone(), json!({"items":[gene()]}));
    envelope::<DiagnosticConditions>(get.clone(), json!({"items":[condition()]}));
    envelope::<DiagnosticMethods>(get, json!({"items":[method()]}));
    envelope::<DiagnosticRegulatory>(
        json!({"source":"fda-pma","id":"P150013","section":"regulatory"}),
        json!(
            {"items":[{"submission":pma_header(),"metadata":{"date_received":"20200101","supplement_type":"","supplement_reason":""},
            "field":"ao_statement","offset":0,"total":1,"text":"Whole synthetic regulatory fragment."}]}
        ),
    );
    envelope::<DiagnosticSupplements>(
        json!({"source":"fda-pma","id":"P150013","section":"supplements"}),
        json!({"items":[pma_header()]}),
    );
    envelope::<DiagnosticCompanion>(
        json!({"source":"fda-cdx","drug_application":"NDA021335"}),
        json!({"items":[companion(false),companion(true)]}),
    );
    assert!(serde_json::from_value::<DiagnosticGenes>(json!({"items":[condition()]})).is_err());
}

#[test]
fn recall_reply_ignores_new_card_and_header_fields() {
    let value = json!({
        "header": {"recall_number": "Z-1234-2025", "future_header": true},
        "future_card": true
    });
    assert_eq!(
        roundtrip::<DeviceRecallCard>(value),
        json!({"header": {"recall_number": "Z-1234-2025"}})
    );
}
