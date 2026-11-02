//! Drug public API contracts, before any command implementation exists.
use biodata::{DrugId, DrugIdentityLevel};
use biomcp_types::{DrugGet, DrugSearch};
use serde_json::json;
#[test]
fn published_ids_roundtrip_without_source_prefixes() {
    for value in [
        "CHEBI:756472",
        "CHEMBL941",
        "EMEA/H/C/000406",
        "NDA021335",
        "741ff3e3-dc1a-45a6-84e5-2481b27131aa",
        "DDInter1744",
        "5",
        "KTUFNOKKBVMGRW-UHFFFAOYSA-N",
    ] {
        let id: DrugId = value.parse().expect("published ID");
        assert_eq!(id.to_string(), value);
        assert_eq!(
            serde_json::from_value::<DrugId>(json!(value)).expect("serde ID"),
            id
        );
    }
    for value in [
        "chembl:CHEMBL941",
        "ema:EMEA/H/C/000406",
        "openfda-label:741ff3e3-dc1a-45a6-84e5-2481b27131aa",
        "",
        "Gleevec tablets",
        "1:foo",
        "CHEBI:",
        "CHEBI::1",
        "CHEBI:/1",
        " CHEMBL941",
    ] {
        assert!(value.parse::<DrugId>().is_err(), "{value}");
        assert!(serde_json::from_value::<DrugId>(json!(value)).is_err());
    }
}
#[test]
fn search_defaults_and_required_get_source_are_explicit() {
    let query: DrugSearch = serde_json::from_value(json!({"query":"Gleevec"})).expect("search");
    assert_eq!(
        serde_json::to_value(query).expect("search JSON")["source"],
        "mychem"
    );
    assert!(serde_json::from_value::<DrugGet>(json!({"input":"CHEMBL941"})).is_err());
    assert!(
        serde_json::from_value::<DrugGet>(json!({"input":"chembl:CHEMBL941", "source":"chembl"}))
            .is_err()
    );
}
#[test]
fn selected_source_checks_its_native_identifier_grammar() {
    use biomcp_types::DrugSource;
    for (source, id) in [
        ("mychem", "KTUFNOKKBVMGRW-UHFFFAOYSA-N"),
        ("mychem", "CHEBI:756472"),
        ("chembl", "CHEMBL941"),
        ("opentargets", "CHEMBL941"),
        ("ema", "EMEA/H/C/000406"),
        ("drugsfda", "NDA021335"),
        ("openfda-label", "741ff3e3-dc1a-45a6-84e5-2481b27131aa"),
        ("ddinter", "DDInter1744"),
        ("civic", "5"),
    ] {
        let selected: DrugSource = source.parse().expect("source");
        let id: DrugId = id.parse().expect("published ID");
        assert!(selected.accepts(&id));
        let request = DrugGet::new(id, selected).expect("native request");
        let json = serde_json::to_value(&request).expect("request JSON");
        assert_eq!(
            serde_json::from_value::<DrugGet>(json).expect("request decode"),
            request
        );
    }
    for (source, id) in [
        ("mychem", "MYCHEM:CHEMBL941"),
        ("mychem", "CHEMBL:CHEMBL941"),
        ("chembl", "Gleevec"),
        ("ema", "CHEMBL941"),
        ("drugsfda", "NDA1"),
        ("ddinter", "1744"),
        ("civic", "0"),
        ("openfda-label", "not-a-uuid"),
    ] {
        assert!(serde_json::from_value::<DrugGet>(json!({"source":source,"input":id})).is_err());
    }
}
#[test]
fn closed_vocabularies_roundtrip_and_refuse_other_sections() {
    use biomcp_types::{DrugSection, DrugSource};
    for name in [
        "ingredient",
        "salt",
        "clinical_formulation",
        "branded_product",
        "spl_set",
        "spl_version",
        "ndc_package",
        "fda_application",
        "fda_product",
        "ema_authorization",
        "ema_application",
        "orphan_designation",
        "unknown",
    ] {
        let value: DrugIdentityLevel = name.parse().expect("identity level");
        assert_eq!(value.to_string(), name);
        assert_eq!(
            serde_json::from_value::<DrugIdentityLevel>(json!(name)).expect("JSON level"),
            value
        );
    }
    for name in [
        "xrefs",
        "targets",
        "indications",
        "civic",
        "label",
        "approvals",
        "regulatory",
        "interactions",
    ] {
        let value: DrugSection = name.parse().expect("section");
        assert_eq!(value.to_string(), name);
        assert_eq!(
            serde_json::from_value::<DrugSection>(json!(name)).expect("JSON section"),
            value
        );
    }
    for name in [
        "mychem",
        "chembl",
        "opentargets",
        "civic",
        "openfda-label",
        "drugsfda",
        "ema",
        "ddinter",
    ] {
        let value: DrugSource = name.parse().expect("source");
        assert_eq!(value.to_string(), name);
        assert_eq!(
            serde_json::from_value::<DrugSource>(json!(name)).expect("JSON source"),
            value
        );
    }
    for name in ["all", "shortage", "safety", "cell_lines", "unknown"] {
        assert!(name.parse::<DrugSection>().is_err());
    }
    assert!("who".parse::<DrugSource>().is_err());
}
#[test]
fn native_text_and_regulator_date_roles_roundtrip_without_interpretation() {
    use biodata::{AuthorizationEvent, DrugLabelSection};
    let event = json!(
        {"record":"EMEA/H/C/000406","jurisdiction":"native jurisdiction","status":"source status","dates":[{"role":"original authorization",
        "date":"2001-11-07"},{"role":"other date","date":"2001-11-06"}],"attribution":{"source":"ema","record":"EMEA/H/C/000406"}}
    );
    let decoded: AuthorizationEvent = serde_json::from_value(event.clone()).expect("event");
    assert_eq!(serde_json::to_value(decoded).expect("event JSON"), event);
    let label = json!({"label":{"id":"741ff3e3-dc1a-45a6-84e5-2481b27131aa",
"attribution":{"source":"openfda-label"}},
"heading":"drug_interactions",
"text":"  original text\nsecond line  "});
    let decoded: DrugLabelSection = serde_json::from_value(label.clone()).expect("label");
    assert_eq!(serde_json::to_value(decoded).expect("label JSON"), label);
}
#[test]
fn section_options_use_existing_continuation_names_and_reject_card_paging() {
    let query = json!(
        {"source":"openfda-label","input":"741ff3e3-dc1a-45a6-84e5-2481b27131aa","section":["label"],"label_section":"drug_interactions",
        "section_offset":10,"section_limit":20}
    );
    let parsed: DrugGet = serde_json::from_value(query.clone()).expect("label continuation");
    assert_eq!(serde_json::to_value(parsed).expect("query JSON"), query);
    for extra in [
        json!({"section_limit":0,"section":["targets"]}),
        json!({"section_limit":51,"section":["targets"]}),
        json!({"section_offset":0}),
        json!({"label_section":"drug_interactions"}),
        json!({"section":["label"]}),
        json!({"product_number":"001"}),
        json!({"text_offset":1}),
    ] {
        let mut value = json!({"source":"chembl", "input":"CHEMBL941"});
        value
            .as_object_mut()
            .expect("query object")
            .extend(extra.as_object().expect("extra object").clone());
        assert!(serde_json::from_value::<DrugGet>(value).is_err());
    }
    for limit in [0, 51] {
        assert!(
            serde_json::from_value::<DrugSearch>(json!({"query":"Gleevec", "limit":limit}))
                .is_err()
        );
    }
}
#[test]
fn civic_therapy_streams_keep_independent_totals_and_profile_context() {
    use biomcp_types::DrugCivic;
    let next = json!({"source":"civic","input":"5","section":["civic"],"section_limit":1,"civic_kind":"evidence","civic_cursor":"MQ"});
    let selected: DrugGet = serde_json::from_value(next.clone()).expect("therapy-wide cursor");
    assert_eq!(selected.civic_profile(), None);
    assert_eq!(
        selected.civic_kind(),
        Some(biomcp_types::CivicClaimKind::Evidence)
    );
    assert_eq!(serde_json::to_value(selected).expect("query JSON"), next);
    let response = json!(
        { "identity":{"id":"5","source":"civic","name":"synthetic therapy","identity_level":"unknown"},"civic":{"state":"data",
        "source":"civic","data":{ "evidence":{"records":[{ "profile":{"record":{"namespace":"unknown","id":"1"},"label":"synthetic profile","compound":false,
        "members":[{"record":{"namespace":"unknown","id":"1"},"label":"synthetic allele","feature":"BRAF","reference_build":"GRCH38"}]},
        "oncokb_missing":["variant"],"claim":{"record":{"namespace":"unknown","id":"EID259"},"kind":"PREDICTIVE","status":"ACCEPTED"},"therapies":[{"id":"5",
        "source":"civic","name":"synthetic therapy","identity_level":"unknown"}] }],"page":{"returned":1,"total":169,"next":next}},
        "assertions":{"page":{"returned":0,"total":1}} }} }
    );
    let decoded: DrugCivic = serde_json::from_value(response.clone()).expect("independent streams");
    assert_eq!(
        serde_json::to_value(decoded).expect("response JSON"),
        response
    );
}
#[test]
fn civic_continuation_refuses_incomplete_or_cross_source_claim_selectors() {
    for selectors in [
        json!({"civic_profile":12}),
        json!({"civic_cursor":"native cursor"}),
        json!({"civic_profile":0,"civic_kind":"evidence"}),
        json!({"civic_profile":2147483648_u64,"civic_kind":"evidence"}),
        json!({"civic_profile":12,"civic_kind":"unknown"}),
        json!({"source":"chembl","input":"CHEMBL941","civic_kind":"evidence"}),
        json!({"section":[],"civic_kind":"evidence"}),
    ] {
        let mut query = json!({"source":"civic","input":"5","section":["civic"]});
        query
            .as_object_mut()
            .expect("query")
            .extend(selectors.as_object().expect("selectors").clone());
        assert!(serde_json::from_value::<DrugGet>(query).is_err());
    }
    let query: DrugGet = serde_json::from_value(json!({"source":"civic", "input":"5", "section":["civic"], "civic_profile":1, "civic_kind":"evidence"}))
        .expect("selected profile");
    assert_eq!(query.civic_profile(), Some(1));
}
#[test]
fn label_candidate_versions_are_native_optional_identity_metadata() {
    let mut value = json!({"id":"211ef2da-2868-4a77-8055-1cb2cd78e24b", "source":"openfda-label", "name":"Synthetic brand", "identity_level":"spl_set"});
    let unversioned: biomcp_types::DrugHeader =
        serde_json::from_value(value.clone()).expect("unversioned");
    assert!(
        serde_json::to_value(unversioned)
            .expect("JSON")
            .get("version")
            .is_none()
    );
    value["version"] = json!("41");
    let versioned: biomcp_types::DrugHeader =
        serde_json::from_value(value.clone()).expect("versioned");
    assert_eq!(serde_json::to_value(versioned).expect("JSON"), value);
    let known = value.clone();
    value["text"] = json!("not an identity field");
    let decoded: biomcp_types::DrugHeader = serde_json::from_value(value).expect("open header");
    assert_eq!(serde_json::to_value(decoded).expect("header JSON"), known);
}
#[test]
fn native_target_identity_does_not_require_an_invented_gene() {
    use biodata::{DrugNativeReference, DrugNativeTarget, DrugTargetRelation};
    let value = json!(
        {"drug":{"id":"CHEMBL1","namespace":"chembl","name":"synthetic drug","identity_level":"unknown"},"native_target":{"namespace":"chembl",
        "id":"CHEMBL999","label":"synthetic target complex","url":"https://example.org/target"},"relation":"native action",
        "mechanism":"source mechanism wording","references":[{"ref_type":"native publication","ref_id":"synthetic-1",
        "ref_url":"https://example.org/reference"}],"attribution":{"source":"chembl"}}
    );
    let parsed: DrugTargetRelation =
        serde_json::from_value(value.clone()).expect("native target without gene");
    assert!(parsed.target().is_none());
    assert_eq!(
        parsed.relation().expect("native relation").as_str(),
        "native action"
    );
    assert_eq!(
        parsed.mechanism().expect("native mechanism").as_str(),
        "source mechanism wording"
    );
    let native = DrugNativeTarget {
        namespace: "chembl".parse().expect("source"),
        id: "CHEMBL999".parse().expect("id"),
        label: Some("synthetic target complex".parse().expect("label")),
        url: Some("https://example.org/target".parse().expect("link")),
    };
    let reference = DrugNativeReference {
        ref_type: "native publication".parse().expect("type"),
        ref_id: Some("synthetic-1".parse().expect("id")),
        ref_url: Some("https://example.org/reference".parse().expect("link")),
    };
    assert_eq!(parsed.references(), &[reference]);
    let rebuilt = DrugTargetRelation::new(
        parsed.drug().clone(),
        None,
        Some(native),
        parsed.relation().cloned(),
        parsed.mechanism().cloned(),
        parsed.references().into(),
        parsed.attribution().clone(),
    )
    .expect("checked target constructor");
    assert_eq!(rebuilt, parsed);
    assert!(
        DrugTargetRelation::new(
            parsed.drug().clone(),
            None,
            None,
            parsed.relation().cloned(),
            None,
            Box::default(),
            parsed.attribution().clone()
        )
        .is_err()
    );
    let gene = biodata::SourceGeneRef::new("source gene".parse().expect("label"), None, None, None)
        .expect("native gene");
    let gene_only = DrugTargetRelation::new(
        parsed.drug().clone(),
        Some(gene.clone()),
        None,
        parsed.relation().cloned(),
        None,
        Box::default(),
        parsed.attribution().clone(),
    )
    .expect("gene-only target");
    assert_eq!(gene_only.target(), Some(&gene));
    let gene_json = serde_json::to_value(&gene_only).expect("gene-only JSON");
    assert!(gene_json.get("native_target").is_none());
    assert_eq!(gene_json["target"], json!({"label":"source gene"}));
    assert_eq!(
        serde_json::from_value::<DrugTargetRelation>(gene_json).expect("gene-only decode"),
        gene_only
    );
    assert_eq!(serde_json::to_value(parsed).expect("target JSON"), value);
    let mut missing = value;
    missing
        .as_object_mut()
        .expect("target object")
        .remove("native_target");
    assert!(serde_json::from_value::<DrugTargetRelation>(missing).is_err());
}
#[test]
fn native_indication_ids_keep_their_own_labels_stage_and_references() {
    use biodata::{DrugDiseaseReference, DrugIndicationClaim, DrugStage};
    let value = json!(
        {"drug":{"id":"CHEMBL1","namespace":"chembl","name":"synthetic drug","identity_level":"unknown"},
        "indication":"source indication wording","disease_references":[{"namespace":"EFO","id":"EFO:synthetic",
        "label":"native EFO wording"},{"namespace":"MeSH","id":"Dsynthetic","label":"different native MeSH wording"}],
        "stage":{"field":"max_phase_for_ind","value":"3.0"},"references":[{"ref_type":"native trial","ref_id":"synthetic-trial",
        "ref_url":"https://example.org/trial"}],"attribution":{"source":"chembl"}}
    );
    let parsed: DrugIndicationClaim =
        serde_json::from_value(value.clone()).expect("native indication");
    assert_eq!(
        parsed.disease_references.first(),
        Some(&DrugDiseaseReference {
            namespace: "EFO".parse().expect("namespace"),
            id: "EFO:synthetic".parse().expect("id"),
            label: Some("native EFO wording".parse().expect("label"))
        })
    );
    assert_eq!(
        parsed.stage,
        Some(DrugStage {
            field: "max_phase_for_ind".into(),
            value: "3.0".into()
        })
    );
    assert_eq!(
        serde_json::to_value(parsed).expect("indication JSON"),
        value
    );
    let stage: DrugStage =
        serde_json::from_value(json!({"field":" native_field ","value":" 3.0 \n"}))
            .expect("unchanged stage strings");
    assert_eq!(stage.field.as_ref(), " native_field ");
    assert_eq!(stage.value.as_ref(), " 3.0 \n");
}
#[test]
fn interaction_keeps_both_bundle_origins_and_refuses_duplicate_attribution() {
    use biodata::DrugInteractionClaim;
    let value = json!(
        {"anchor":{"id":"DDInter1","namespace":"ddinter","name":"synthetic anchor","identity_level":"unknown"},"partner":{"id":"DDInter2",
        "namespace":"ddinter","name":"synthetic partner","identity_level":"unknown"},"level":"native level","attribution":{"source":"ddinter",
        "record":"B-synthetic"},"additional_attributions":[{"source":"ddinter","record":"L-synthetic"}]}
    );
    let parsed: DrugInteractionClaim =
        serde_json::from_value(value.clone()).expect("both bundle origins");
    assert_eq!(parsed.anchor().id.to_string(), "DDInter1");
    assert_eq!(parsed.partner().id.to_string(), "DDInter2");
    assert_eq!(parsed.level().map(|v| v.as_str()), Some("native level"));
    assert_eq!(
        parsed
            .attribution()
            .record
            .as_ref()
            .expect("primary")
            .as_str(),
        "B-synthetic"
    );
    let additional: Box<[biodata::Attribution]> = serde_json::from_value(
        serde_json::to_value(&parsed).expect("serialize interaction")["additional_attributions"]
            .clone(),
    )
    .expect("checked secondary origins");
    assert_eq!(
        additional[0]
            .record
            .as_ref()
            .expect("secondary record")
            .as_str(),
        "L-synthetic"
    );
    let rebuilt = DrugInteractionClaim::new(
        parsed.anchor().clone(),
        parsed.partner().clone(),
        parsed.level().cloned(),
        parsed.attribution().clone(),
        additional,
    )
    .expect("checked interaction constructor");
    assert_eq!(rebuilt, parsed);
    assert!(
        DrugInteractionClaim::new(
            parsed.anchor().clone(),
            parsed.partner().clone(),
            parsed.level().cloned(),
            parsed.attribution().clone(),
            vec![parsed.attribution().clone()].into()
        )
        .is_err()
    );
    assert_eq!(
        serde_json::to_value(parsed).expect("interaction JSON"),
        value
    );
    let mut duplicate = value.clone();
    duplicate["additional_attributions"] = json!([value["attribution"].clone()]);
    assert!(serde_json::from_value::<DrugInteractionClaim>(duplicate).is_err());
    let mut duplicate = value.clone();
    duplicate["additional_attributions"] = json!([
        value["additional_attributions"][0].clone(),
        value["additional_attributions"][0].clone()
    ]);
    assert!(serde_json::from_value::<DrugInteractionClaim>(duplicate).is_err());
    let mut legacy = value;
    legacy
        .as_object_mut()
        .expect("interaction object")
        .remove("additional_attributions");
    let parsed: DrugInteractionClaim =
        serde_json::from_value(legacy.clone()).expect("primary origin alone");
    assert_eq!(
        serde_json::to_value(parsed).expect("legacy interaction JSON"),
        legacy
    );
}
#[test]
#[cfg(feature = "schema")]
fn changed_native_record_responses_publish_version_three_and_checked_target_schema() {
    use biodata::DrugTargetRelation;
    use biomcp_types::{DrugIndications, DrugInteractions, DrugTargets, ResponseBody};
    assert_eq!(DrugTargets::SCHEMA, "biomcp.drug-targets/8");
    assert_eq!(DrugIndications::SCHEMA, "biomcp.drug-indications/8");
    assert_eq!(DrugInteractions::SCHEMA, "biomcp.drug-interactions/8");
    let validator =
        jsonschema::draft202012::new(&schemars::schema_for!(DrugTargetRelation).to_value())
            .expect("target schema");
    let base = json!(
        {"drug":{"id":"CHEMBL1","namespace":"chembl","name":"synthetic drug","identity_level":"unknown"},"relation":"source action",
        "attribution":{"source":"chembl"}}
    );
    for selectors in [
        json!({}),
        json!({"target":null}),
        json!({"native_target":null}),
        json!({"target":null,"native_target":null}),
    ] {
        let mut invalid = base.clone();
        invalid
            .as_object_mut()
            .expect("record")
            .extend(selectors.as_object().expect("selectors").clone());
        assert!(!validator.is_valid(&invalid));
        assert!(serde_json::from_value::<DrugTargetRelation>(invalid).is_err());
    }
    for selectors in [
        json!({"target":{"label":"source gene"}}),
        json!({"native_target":{"namespace":"chembl","id":"CHEMBL999"}}),
        json!({"target":{"label":"source gene"},"native_target":{"namespace":"chembl","id":"CHEMBL999"}}),
    ] {
        let mut valid = base.clone();
        valid
            .as_object_mut()
            .expect("record")
            .extend(selectors.as_object().expect("selectors").clone());
        assert!(validator.is_valid(&valid));
        assert!(serde_json::from_value::<DrugTargetRelation>(valid).is_ok());
    }
}
#[test]
fn chembl_name_selector_is_explicit_and_source_scoped() {
    use biomcp_types::DrugNameSelector;
    let default: DrugSearch =
        serde_json::from_value(json!({"query":"imatinib","source":"chembl"})).unwrap();
    assert_eq!(default.name_selector(), DrugNameSelector::Preferred);
    let synonym: DrugSearch = serde_json::from_value(
        json!({"query":"Enhertu","source":"chembl","name_selector":"synonym"}),
    )
    .unwrap();
    assert_eq!(synonym.name_selector(), DrugNameSelector::Synonym);
    for source in [
        "mychem",
        "opentargets",
        "civic",
        "openfda-label",
        "ema",
        "drugsfda",
        "ddinter",
    ] {
        for selector in ["preferred", "synonym"] {
            assert!(
                serde_json::from_value::<DrugSearch>(
                    json!({"query":"imatinib","source":source,"name_selector":selector})
                )
                .is_err()
            );
        }
    }
    for source in ["chembl", "mychem"] {
        assert!(
            serde_json::from_value::<DrugSearch>(
                json!({"query":"imatinib","source":source,"name_selector":null})
            )
            .is_err()
        );
    }
    assert!(
        serde_json::from_value::<DrugSearch>(
            json!({"query":"imatinib","source":"chembl","name_selector":"automatic"})
        )
        .is_err()
    );
}

#[test]
#[cfg(feature = "schema")]
fn drug_recall_name_omission_and_present_values_match_the_schema() {
    let schema = schemars::schema_for!(DrugSearch).to_value();
    let validator = jsonschema::draft202012::new(&schema).expect("drug search schema");
    for (query, valid) in [
        (
            json!({"source":"fda-drug-recall","firm":"Synthetic maker"}),
            true,
        ),
        (
            json!({"source":"fda-drug-recall","query":"Synthetic name"}),
            true,
        ),
        (json!({"source":"fda-drug-recall","query":null}), false),
    ] {
        assert_eq!(validator.is_valid(&query), valid, "{query}");
        assert_eq!(
            serde_json::from_value::<DrugSearch>(query.clone()).is_ok(),
            valid,
            "{query}"
        );
    }
    for source in biomcp_types::DrugSource::ALL {
        if source != biomcp_types::DrugSource::FdaDrugRecall {
            assert!(
                serde_json::from_value::<DrugSearch>(json!({"source":source})).is_err(),
                "{source}"
            );
        }
    }
}
