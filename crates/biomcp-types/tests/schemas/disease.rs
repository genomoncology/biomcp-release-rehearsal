use super::{Contract, generate};
use biodata::*;
use biomcp_types::*;
pub(super) fn registry() -> Vec<Contract> {
    contracts! {
        DiseaseAssociation => ("disease-association", "biodata.disease-association/7"),
        DiseaseRelation => ("disease-relation", "biodata.disease-relation/4"),
        DiseaseEndpoint => ("disease-endpoint", "biodata.disease-endpoint/3"),
        DiseaseSynonym => ("disease-synonym", "biodata.disease-synonym/3"),
        DiseaseStatistic => ("disease-statistic", "biodata.disease-statistic/4"),
        DiseaseOntology => ("disease-ontology", "biomcp.disease-ontology/1"),
    }
}

#[test]
fn request_metadata_keeps_positional_text_single_ontology_and_section_paging() {
    let search = generate::<DiseaseSearch>();
    assert_eq!(search["properties"]["text"]["x-positional"], 0);
    assert_eq!(search["properties"]["ontology"]["default"], "mondo");
    assert_eq!(
        search["$defs"]["DiseaseOntology"]["enum"],
        serde_json::json!(["mondo", "doid", "ncit"])
    );
    let get = generate::<DiseaseGet>();
    let fields = get["properties"].as_object().unwrap();
    assert!(fields.contains_key("section_limit"));
    assert!(fields.contains_key("section_offset"));
    assert!(!fields.contains_key("limit"));
    assert!(!fields.contains_key("offset"));
    assert_eq!(fields["id"]["x-positional"], 0);
}

fn envelope<T>(
    query: serde_json::Value,
    body: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: ResponseBody + schemars::JsonSchema + serde::de::DeserializeOwned,
    T::Query: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let body: T = serde_json::from_value(body)?;
    let query = serde_json::from_value(query)?;
    let value = serde_json::to_value(Envelope::new(Status::Complete, query, Some(body)))?;
    assert_eq!(
        value.get("schema").and_then(serde_json::Value::as_str),
        Some(T::SCHEMA),
        "Disease schemas must retain their versioned envelope contract"
    );
    let validator = jsonschema::draft202012::new(&generate::<Envelope<T>>())?;
    assert!(validator.is_valid(&value), "{value}");
    Ok(())
}

#[test]
fn all_ten_synthetic_response_envelopes_follow_their_generated_schema() {
    use serde_json::json;
    let search = json!({"text":"synthetic cancer"});
    let get = json!({"id":"MONDO:0007254"});
    let empty = json!({"items":[],"components":[]});
    envelope::<DiseasePage>(search, empty.clone()).unwrap();
    envelope::<DiseaseCard>(
        get.clone(),
        json!(
            {"requested_id":"MONDO:0007254","returned_id":"MONDO:0007254","concept":{"id":"MONDO:0007254","label":"synthetic cancer"},
            "definition":"Complete synthetic definition.","obsolete":true,"consider":["MONDO:0800444","MONDO:0800445"],
            "attribution":{"source":"synthetic"},"sections":["synonyms","xrefs","hierarchy"],"components":[]}
        ),
    )
    .unwrap();
    envelope::<DiseaseSynonyms>(
        get.clone(),
        json!(
            {"items":[{"value":"synthetic synonym","scope":"oboInOwl:hasExactSynonym","language":"en","attribution":{"source":"synthetic"}}],
            "components":[]}
        ),
    )
    .unwrap();
    let relations = json!(
        {"items":[{"subject":{"id":"MONDO:0007254","label":"synthetic"},"predicate":"skos:closeMatch","object":{"id":"DOID:1612",
        "label":"native target"},"attribution":{"source":"synthetic"}}],"components":[]}
    );
    envelope::<DiseaseXrefs>(get.clone(), relations.clone()).unwrap();
    envelope::<DiseaseHierarchy>(get.clone(), relations).unwrap();
    envelope::<DiseaseGenes>(get.clone(), empty.clone()).unwrap();
    envelope::<DiseasePhenotypes>(get, empty.clone()).unwrap();
    envelope::<DiseaseSites>(
        json!({"source":"seer"}),
        json!({"items":[{"code":"47","label":"Synthetic site","active":true}],"components":[]}),
    )
    .unwrap();
    envelope::<DiseaseIncidence>(
        json!({"id":"47","source":"seer","section":"incidence","period":"5"}),
        empty.clone(),
    )
    .unwrap();
    envelope::<DiseaseSurvival>(
        json!({"id":"55","source":"seer","section":"survival","interval":"5"}),
        empty,
    )
    .unwrap();
}
