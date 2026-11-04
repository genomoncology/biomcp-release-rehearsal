//! Feature: phenotype-types. Public checked request and neutral record contracts.
use biodata::HpId;
use biomcp_types::Paged;
use biomcp_types::{PhenotypeGet, PhenotypeSearch, PhenotypeSource};
use serde_json::json;

#[test]
fn hp_identity_round_trips_and_rejects_other_namespaces() {
    for value in ["HP:0001250", "HP:0000000", "HP:9999999"] {
        let id: HpId = value.parse().unwrap();
        assert_eq!(id.to_string(), value);
        assert_eq!(serde_json::from_value::<HpId>(json!(value)).unwrap(), id);
    }
    for value in [
        "HP:1250",
        "HP:00001250",
        "hp:0001250",
        "MONDO:0001250",
        "HGNC:1",
        " HP:0001250",
        "HP:00012a0",
    ] {
        assert!(value.parse::<HpId>().is_err(), "{value}");
    }
}

#[test]
fn source_and_section_pairings_validate_before_any_adapter() {
    let search: PhenotypeSearch = serde_json::from_value(json!({"text":"seizure"})).unwrap();
    assert_eq!(search.source(), PhenotypeSource::Monarch);
    assert_eq!(search.limit(), 5);
    for section in [
        None,
        Some("synonyms"),
        Some("xrefs"),
        Some("hierarchy"),
        Some("diseases"),
        Some("genes"),
    ] {
        let mut value = json!({"id":"HP:0001250"});
        if let Some(section) = section {
            value
                .as_object_mut()
                .unwrap()
                .insert("section".into(), json!(section));
        }
        let get: PhenotypeGet = serde_json::from_value(value.clone()).unwrap();
        let association = matches!(section, Some("diseases" | "genes"));
        let monarch = association || section == Some("hierarchy");
        assert_eq!(
            get.source(),
            if monarch {
                PhenotypeSource::Monarch
            } else {
                PhenotypeSource::Hpo
            }
        );
        assert_eq!(get.section_limit(), if association { 1 } else { 5 });
        for source in ["hpo", "monarch"] {
            value
                .as_object_mut()
                .unwrap()
                .insert("source".into(), json!(source));
            assert_eq!(
                serde_json::from_value::<PhenotypeGet>(value.clone()).is_ok(),
                source == "monarch" || !monarch
            );
        }
    }
    let card = PhenotypeGet::new("HP:0001250".parse().unwrap());
    assert!(
        card.with_section(biomcp_types::PhenotypeSection::Hierarchy)
            .is_err()
    );
    assert!(
        card.with_section(biomcp_types::PhenotypeSection::Synonyms)
            .is_ok()
    );
    for bad in [
        json!({"text":"x","source":"hpo"}),
        json!({"text":" "}),
        json!({"text":"x","limit":0}),
        json!({"text":"x","limit":51}),
        json!({"text":"x","offset":-1}),
    ] {
        assert!(serde_json::from_value::<PhenotypeSearch>(bad).is_err());
    }
    for bad in [
        json!({"id":"MONDO:1"}),
        json!({"id":"HP:0001250","section":"similarity"}),
        json!({"id":"HP:0001250","section_limit":0}),
        json!({"id":"HP:0001250","source":null}),
    ] {
        assert!(serde_json::from_value::<PhenotypeGet>(bad).is_err());
    }
}

#[test]
fn native_cards_keep_empty_values_independent_sources_and_count_scopes() {
    use biodata::{PhenotypeCard, PhenotypeSynonym};
    let hpo = json!(
        {"source":"hpo","requested_id":"HP:0001250","returned_id":"HP:0001250","label":"Seizure","definition":"","comment":"",
        "hpo_descendant_count":346,"attribution":{"source":"hpo"}}
    );
    let card: PhenotypeCard = serde_json::from_value(hpo.clone()).unwrap();
    assert_eq!(serde_json::to_value(card).unwrap(), hpo);
    let absent: PhenotypeCard = serde_json::from_value(json!(
        {"source":"hpo","requested_id":"HP:0001250","returned_id":"HP:0001250","label":"Seizure","attribution":{"source":"hpo"}}
    ))
    .unwrap();
    assert!(
        serde_json::to_value(absent)
            .unwrap()
            .get("definition")
            .is_none()
    );
    let monarch = json!(
        {"source":"monarch","requested_id":"HP:0001676","returned_id":"HP:0001676","label":"obsolete source label","deprecated":true,
        "monarch_descendant_count":1,"association_counts":[{"key":"biolink:DiseaseToPhenotypicFeatureAssociation",
        "category":"biolink:DiseaseToPhenotypicFeatureAssociation","label":"Disease to Phenotype","count":5350,"count_direct":2561}],
        "attribution":{"source":"monarch"}}
    );
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<PhenotypeCard>(monarch.clone()).unwrap())
            .unwrap(),
        monarch
    );
    let mut merged = hpo.clone();
    merged
        .as_object_mut()
        .unwrap()
        .insert("deprecated".into(), json!(false));
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<PhenotypeCard>(merged).unwrap()).unwrap(),
        hpo
    );
    for relationship in [
        None,
        Some("exact"),
        Some("broad"),
        Some("narrow"),
        Some("related"),
    ] {
        let mut row = json!({"value":"","attribution":{"source":"synthetic"}});
        if let Some(r) = relationship {
            row.as_object_mut()
                .unwrap()
                .insert("relationship".into(), json!(r));
        }
        assert_eq!(
            serde_json::to_value(serde_json::from_value::<PhenotypeSynonym>(row.clone()).unwrap())
                .unwrap(),
            row
        );
    }
}

#[test]
fn associations_reuse_native_endpoints_and_never_deserialize_as_statements() {
    use biodata::{DiseaseAssociation, Statement};
    let row = json!(
        {"requested_id":"HP:0001250","subject":{"id":"HGNC:29529","label":"synthetic gene"},"subject_taxon":"NCBITaxon:9606",
        "object":{"id":"HP:0001250","label":"synthetic term"},"predicate":"biolink:has_phenotype","attribution":{"source":"synthetic"},
        }
    );
    let association: DiseaseAssociation = serde_json::from_value(row.clone()).unwrap();
    assert_eq!(serde_json::to_value(association).unwrap(), row);
    assert!(serde_json::from_value::<Statement>(row).is_err());
}

#[test]
fn continuations_keep_checked_selectors_and_explicit_limits() {
    let search: PhenotypeSearch =
        serde_json::from_value(json!({"text":" native text ","limit":50,"offset":5})).unwrap();
    let next = search.at_offset(55);
    assert_eq!(next.text(), " native text ");
    assert_eq!(next.limit(), 50);
    assert_eq!(next.offset(), 55);
    let get: PhenotypeGet = serde_json::from_value(
        json!({"id":"HP:0001250","source":"monarch","section":"genes","section_limit":2}),
    )
    .unwrap();
    let next = get.at_section_offset(2);
    assert_eq!(next.id().as_str(), "HP:0001250");
    assert_eq!(next.section_limit(), 2);
    assert_eq!(next.section_offset(), 2);
    assert_eq!(next.source(), PhenotypeSource::Monarch);
}

#[cfg(feature = "schema")]
fn envelope<T>(
    query: serde_json::Value,
    body: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: biomcp_types::ResponseBody + schemars::JsonSchema + serde::de::DeserializeOwned,
    T::Query: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let body: T = serde_json::from_value(body)?;
    let query = serde_json::from_value(query)?;
    let value = serde_json::to_value(biomcp_types::Envelope::new(
        biomcp_types::Status::Complete,
        query,
        Some(body),
    ))?;
    assert_eq!(
        value.get("schema").and_then(serde_json::Value::as_str),
        Some(T::SCHEMA),
        "Phenotype replies must retain their versioned envelope contract"
    );
    let schema = schemars::schema_for!(biomcp_types::Envelope<T>);
    let schema = serde_json::to_value(schema)?;
    assert!(
        jsonschema::draft202012::new(&schema)?.is_valid(&value),
        "{value}"
    );
    Ok(())
}

#[cfg(feature = "schema")]
#[test]
fn seven_public_envelopes_keep_source_cards_and_native_section_rows() {
    use biomcp_types::*;
    let get = json!({"id":"HP:0001250"});
    envelope::<PhenotypePage>(
        json!({"text":"seizure"}),
        json!(
            {"items":[{"id":"HP:0001250","label":"Seizure","match_score":1.5,"matched_field":"name","match_type":"exact",
            "attribution":{"source":"synthetic"}}],"components":[]}
        ),
    )
    .unwrap();
    envelope::<PhenotypeTermCard>(
        get.clone(),
        json!(
            {"term":{"source":"hpo","requested_id":"HP:0001250","returned_id":"HP:0001250","label":"Seizure","definition":"",
            "attribution":{"source":"synthetic"}},"sections":["synonyms","xrefs"],"components":[]}
        ),
    )
    .unwrap();
    envelope::<PhenotypeSynonyms>(
        get.clone(),
        json!({"items":[{"value":"","relationship":"related","attribution":{"source":"synthetic"}}],"components":[]}),
    )
    .unwrap();
    envelope::<PhenotypeXrefs>(
        get.clone(),
        json!({"items":[{"id":"Fyler:4335","attribution":{"source":"synthetic"}}],"components":[]}),
    )
    .unwrap();
    envelope::<PhenotypeHierarchy>(
        json!({"id":"HP:0001250","section":"hierarchy"}),
        json!({"items":[{"id":"HP:0000001","label":"All","relation":"super_classes","attribution":{"source":"synthetic"}}],"components":[]}),
    )
    .unwrap();
    let empty = json!({"items":[],"components":[],"direct":true});
    envelope::<PhenotypeDiseases>(
        json!({"id":"HP:0001250","section":"diseases"}),
        empty.clone(),
    )
    .unwrap();
    envelope::<PhenotypeGenes>(json!({"id":"HP:0001250","section":"genes"}), empty).unwrap();
}
