//! Disease public wire contracts precede provider implementations.
use biodata::{DiseaseAssociation, Statement};
use biomcp_types::Paged;
use biomcp_types::{DiseaseCard, DiseaseGet, DiseaseOntology, DiseaseSearch};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn roundtrip<T: DeserializeOwned + Serialize>(value: Value) -> Result<Value, serde_json::Error> {
    serde_json::to_value(serde_json::from_value::<T>(value)?)
}

#[test]
fn discovery_defaults_to_one_mondo_ontology_and_refuses_unknown_choices() {
    let value = roundtrip::<DiseaseSearch>(json!({"text":"breast cancer"})).unwrap();
    assert_eq!(value["ontology"], "mondo");
    assert_eq!(value["source"], "ols");
    for ontology in ["mondo", "doid", "ncit"] {
        assert_eq!(
            roundtrip::<DiseaseOntology>(json!(ontology)).unwrap(),
            ontology
        );
    }
    for ontology in ["mondo,doid", "mesh", "MONDO", ""] {
        assert!(serde_json::from_value::<DiseaseOntology>(json!(ontology)).is_err());
    }
}

#[test]
fn exact_identity_and_statistics_selectors_cannot_be_mixed() {
    for id in [
        "MONDO:0007254",
        "DOID:1612",
        "NCIT:C4872",
        "OncoTree:BRCA",
        "ICD10CM:C50.9",
    ] {
        assert_eq!(roundtrip::<DiseaseGet>(json!({"id":id})).unwrap()["id"], id);
    }
    for value in [
        json!({"id":"MONDO:0007254","source":"seer","section":"incidence"}),
        json!({"id":"47","source":"seer","section":"survival","period":"5"}),
        json!({"id":"47","source":"seer","section":"survival","ontology":"mondo"}),
        json!({"id":"MONDO:0007254","ontology":"mondo"}),
        json!({"id":"breast cancer"}),
        json!({"id":"MONDO:0007254","section":"all"}),
        json!({"id":"MONDO:0007254","limit":5}),
    ] {
        assert!(
            serde_json::from_value::<DiseaseGet>(value.clone()).is_err(),
            "{value}"
        );
    }
    assert!(
        serde_json::from_value::<DiseaseGet>(
            json!({"id":"47","source":"seer","section":"incidence","period":"5"})
        )
        .is_ok()
    );
}

#[test]
fn card_omits_unrecognized_associations_statistics_and_related_entities() {
    let card = json!(
        {"requested_id":"MONDO:0007254","returned_id":"MONDO:0007254","concept":{"id":"MONDO:0007254","label":"breast cancer"},
        "attribution":{"source":"synthetic"},"replaced_by":null}
    );
    let mut expected = card.clone();
    expected.as_object_mut().unwrap().remove("replaced_by");
    assert_eq!(roundtrip::<DiseaseCard>(card.clone()).unwrap(), expected);
    for field in [
        "genes",
        "phenotypes",
        "incidence",
        "survival",
        "trials",
        "drugs",
    ] {
        let mut invalid = card.clone();
        invalid[field] = json!([]);
        assert_eq!(roundtrip::<DiseaseCard>(invalid).unwrap(), expected);
    }
}

#[test]
fn association_keeps_parent_request_and_native_child_without_becoming_statement() {
    let row = json!(
        {"requested_id":"MONDO:0019003","subject":{"id":"HGNC:9967","label":"RET"},"object":{"id":"MONDO:0008234","label":"MEN2A"},
        "predicate":"biolink:gene_associated_with_condition","original_subject":"NCBIGene:5979","original_object":"Orphanet:247698",
        "aggregator_knowledge_source":["infores:monarchinitiative"],"attribution":{"source":"synthetic"},
        "object_closure":["MONDO:0019003"]}
    );
    assert_eq!(roundtrip::<DiseaseAssociation>(row.clone()).unwrap(), row);
    assert!(serde_json::from_value::<Statement>(row).is_err());
}

#[test]
fn native_statistics_keep_requested_and_returned_dimensions_and_missing_observations() {
    let value = json!(
        {"requested_site":"55","requested_selectors":[{"name":"dataset","code":"4"}],"returned_selectors":[{"name":"site","code":"55"},
        {"name":"year_range","code":"31"}],"year":2022,"measure":"rel_rate","modeled":{"value":{"value":93.690345}},
        "attribution":{"source":"synthetic"}}
    );
    let result = roundtrip::<biodata::DiseaseStatistic>(value.clone()).unwrap();
    assert_eq!(result, value);
    assert!(result.get("observed").is_none());
    assert_eq!(result["requested_selectors"][0]["code"], "4");
    assert_eq!(result["returned_selectors"][1]["code"], "31");
}

#[test]
fn every_source_and_named_section_roundtrips_with_closed_spellings() {
    use biomcp_types::{DiseaseSection, DiseaseSource};
    for spelling in ["ols", "mydisease", "monarch", "seer"] {
        let source: DiseaseSource = serde_json::from_value(json!(spelling)).unwrap();
        assert_eq!(source.as_str(), spelling);
        assert_eq!(
            roundtrip::<DiseaseSource>(json!(spelling)).unwrap(),
            spelling
        );
    }
    for spelling in [
        "synonyms",
        "xrefs",
        "hierarchy",
        "genes",
        "phenotypes",
        "incidence",
        "survival",
    ] {
        let section: DiseaseSection = serde_json::from_value(json!(spelling)).unwrap();
        assert_eq!(section.as_str(), spelling);
        assert_eq!(
            roundtrip::<DiseaseSection>(json!(spelling)).unwrap(),
            spelling
        );
    }
    for spelling in ["all", "clinical_features", "pathways", "civic", "shortages"] {
        assert!(serde_json::from_value::<DiseaseSection>(json!(spelling)).is_err());
        assert!(serde_json::from_value::<DiseaseSource>(json!(spelling)).is_err());
    }
}

#[test]
fn association_component_terms_stay_bound_to_the_native_row() {
    let row = json!(
        {"association":{"requested_id":"MONDO:0019003","subject":{"id":"HGNC:9967","label":"RET"},"object":{"id":"MONDO:0008234",
        "label":"MEN2A"},"predicate":"biolink:gene_associated_with_condition","attribution":{"source":"synthetic","record":"edge-1",
        "version":"2026-01-01"},"object_closure":["MONDO:0019003"]},
        "components":[{"component":"hpo","terms":{"usage":"attribution","name":"Synthetic attribution terms",
        "url":"https://example.invalid/terms"}}]}
    );
    assert_eq!(
        roundtrip::<biomcp_types::DiseaseAssociationItem>(row.clone()).unwrap(),
        row
    );
    assert!(serde_json::from_value::<DiseaseAssociation>(row).is_err());
}

#[test]
fn candidate_header_preserves_publisher_obsolete_state_without_guessing() {
    let mut header = json!({"concept":{"id":"MONDO:0007607","label":"synthetic obsolete concept"},"ontology":"mondo","attribution":{"source":"synthetic"}});
    assert_eq!(
        roundtrip::<biomcp_types::DiseaseHeader>(header.clone()).unwrap(),
        header
    );
    for state in [true, false] {
        header["obsolete"] = json!(state);
        assert_eq!(
            roundtrip::<biomcp_types::DiseaseHeader>(header.clone()).unwrap(),
            header
        );
    }
}

#[test]
fn disease_page_continuations_keep_every_native_selector() {
    let search: DiseaseSearch =
        serde_json::from_value(json!({"text":"MF","ontology":"doid","limit":2})).unwrap();
    assert_eq!(search.text().unwrap(), "MF");
    assert_eq!(search.source(), biomcp_types::DiseaseSource::Ols);
    assert_eq!(search.ontology(), Some(DiseaseOntology::Doid));
    let next = search.at_offset(2);
    assert_eq!((next.limit(), next.offset()), (2, 2));
    let get: DiseaseGet = serde_json::from_value(json!(
        {"id":"47","source":"seer","section":"incidence","section_limit":2,"sex":"1","race":"1","age_range":"1","dataset":"4","period":"5"}
    ))
    .unwrap();
    let resolved = get
        .with_demographics(
            "1".parse().unwrap(),
            "1".parse().unwrap(),
            "1".parse().unwrap(),
        )
        .unwrap();
    let next = resolved.at_section_offset(2);
    assert_eq!(next.id().as_str(), "47");
    assert_eq!(next.source(), biomcp_types::DiseaseSource::Seer);
    assert_eq!(
        next.section(),
        Some(biomcp_types::DiseaseSection::Incidence)
    );
    assert_eq!((next.section_limit(), next.section_offset()), (2, 2));
    assert_eq!(next.sex().unwrap().as_str(), "1");
    assert_eq!(next.race().unwrap().as_str(), "1");
    assert_eq!(next.age_range().unwrap().as_str(), "1");
    assert_eq!(next.dataset().unwrap().as_str(), "4");
    assert_eq!(next.period().unwrap().as_str(), "5");
    assert!(next.interval().is_none());
    assert!(next.ontology().is_none());
}

#[test]
fn native_alias_proof_keeps_unlabeled_endpoints_and_raw_description() {
    let relation = json!(
        {"subject":{"id":"DOID:3910"},"predicate":"oboInOwl:hasDbXref","object":{"id":"MONDO:0005061"},
        "description":"  Native source text  ","attribution":{"source":"synthetic"}}
    );
    assert_eq!(
        roundtrip::<biodata::DiseaseRelation>(relation.clone()).unwrap(),
        relation
    );
    let mut invalid = relation.clone();
    invalid["object"] = json!({"label":"unidentified"});
    assert!(serde_json::from_value::<biodata::DiseaseRelation>(invalid).is_err());
    let card = json!(
        {"requested_id":"DOID:3910","returned_id":"MONDO:0005061","alias":relation,"concept":{"id":"MONDO:0005061","label":"synthetic"},
        "replaced_by":null,"attribution":{"source":"synthetic"},}
    );
    let mut expected = card.clone();
    expected.as_object_mut().unwrap().remove("replaced_by");
    assert_eq!(roundtrip::<DiseaseCard>(card).unwrap(), expected);
}

#[test]
fn association_aggregators_keep_native_null_and_omit_empty_lists() {
    let base = json!(
        {"requested_id":"MONDO:0019003","subject":{"id":"HGNC:9967","label":"Synthetic"},"object":{"id":"MONDO:0007958",
        "label":"Synthetic child"},"predicate":"biolink:associated_with","attribution":{"source":"synthetic"},
        }
    );
    assert_eq!(roundtrip::<DiseaseAssociation>(base.clone()).unwrap(), base);
    for sources in [
        json!(null),
        json!([]),
        json!(["infores:monarchinitiative", "infores:hpo-annotations"]),
    ] {
        let mut row = base.clone();
        let empty = sources == json!([]);
        row["aggregator_knowledge_source"] = sources;
        let output = roundtrip::<DiseaseAssociation>(row.clone()).unwrap();
        if empty {
            row.as_object_mut()
                .unwrap()
                .remove("aggregator_knowledge_source");
        }
        assert_eq!(output, row);
    }
    for sources in [json!("infores:monarchinitiative"), json!(["invalid"])] {
        let mut row = base.clone();
        row["aggregator_knowledge_source"] = sources;
        assert!(serde_json::from_value::<DiseaseAssociation>(row).is_err());
    }
}

#[test]
fn association_native_numbers_are_independent_and_never_derived() {
    let base = json!(
        {"requested_id":"MONDO:0019003","subject":{"id":"MONDO:0019003","label":"Synthetic"},"object":{"id":"HP:0000001",
        "label":"Synthetic feature"},"predicate":"biolink:has_phenotype","attribution":{"source":"synthetic"},
        }
    );
    for (field, value) in [
        ("has_count", json!(0)),
        ("has_total", json!(35)),
        ("has_percentage", json!(77.14286)),
        ("has_quotient", json!(0.7714286)),
    ] {
        let mut row = base.clone();
        row[field] = value;
        assert_eq!(roundtrip::<DiseaseAssociation>(row.clone()).unwrap(), row);
    }
    let mut invalid = base;
    invalid["has_count"] = json!(-1);
    assert!(serde_json::from_value::<DiseaseAssociation>(invalid).is_err());
}

#[test]
fn association_references_allow_native_urls_without_invented_labels() {
    let row = json!(
        {"requested_id":"MONDO:0019003","subject":{"id":"MONDO:0019003","label":"Synthetic"},"object":{"id":"HP:0000001",
        "label":"Synthetic feature"},"predicate":"biolink:has_phenotype","attribution":{"source":"synthetic"},
        "evidence":[{"id":"ECO:0000501","url":"http://purl.obolibrary.org/obo/ECO_0000501"}],"publications":[{"id":"PMID:1",
        "url":"http://identifiers.org/pubmed/1"}],"qualifiers":[{"field":"frequency_qualifier","id":"HP:0040283",
        "label":"Synthetic frequency"}],}
    );
    assert_eq!(roundtrip::<DiseaseAssociation>(row.clone()).unwrap(), row);
}

#[test]
fn source_scoped_defaults_preserve_explicit_limits_and_reject_null_or_zero() {
    for (source, section, expected) in [
        ("monarch", "genes", 2),
        ("monarch", "phenotypes", 1),
        ("mydisease", "synonyms", 5),
        ("ols", "xrefs", 5),
    ] {
        let base = json!({"id":"MONDO:0019003","source":source,"section":section});
        let resolved = roundtrip::<DiseaseGet>(base.clone()).unwrap();
        assert_eq!(resolved["section_limit"], expected);
        for limit in [json!(1), json!(5), json!(50)] {
            let mut explicit = base.clone();
            explicit["section_limit"] = limit.clone();
            assert_eq!(
                roundtrip::<DiseaseGet>(explicit).unwrap()["section_limit"],
                limit
            );
        }
        for limit in [json!(null), json!(0), json!(51)] {
            let mut invalid = base.clone();
            invalid["section_limit"] = limit;
            assert!(serde_json::from_value::<DiseaseGet>(invalid).is_err());
        }
    }
}

#[test]
fn association_evidence_omits_empty_lists_and_keeps_populated_terms() {
    let base = json!(
        {"requested_id":"MONDO:0019003","subject":{"id":"MONDO:0019003","label":"Synthetic"},"object":{"id":"HP:0000001",
        "label":"Synthetic feature"},"predicate":"biolink:has_phenotype","attribution":{"source":"synthetic"},
        }
    );
    assert_eq!(roundtrip::<DiseaseAssociation>(base.clone()).unwrap(), base);
    let mut unavailable = base.clone();
    unavailable["evidence"] = json!(null);
    assert_eq!(roundtrip::<DiseaseAssociation>(unavailable).unwrap(), base);
    for evidence in [
        json!([]),
        json!([{"id":"ECO:0000501","label":"Synthetic evidence"}]),
    ] {
        let mut row = base.clone();
        let empty = evidence == json!([]);
        row["evidence"] = evidence;
        let output = roundtrip::<DiseaseAssociation>(row.clone()).unwrap();
        if empty {
            row.as_object_mut().unwrap().remove("evidence");
        }
        assert_eq!(output, row);
    }
}
