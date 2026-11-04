//! Temporal field meanings stay consistent across every registered schema.
use super::generate;
use biomcp_types::SourceDateTimeValue;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Types = BTreeMap<String, BTreeSet<String>>;

fn collect(node: &Value, types: &mut Types) {
    if let Some(properties) = node.get("properties").and_then(Value::as_object) {
        for name in ["source_updated", "updated", "fetched", "retrieved"] {
            if let Some(property) = properties.get(name) {
                property_types(property, types.entry(name.into()).or_default());
            }
        }
    }
    match node {
        Value::Object(values) => values.values().for_each(|value| collect(value, types)),
        Value::Array(values) => values.iter().for_each(|value| collect(value, types)),
        _ => {}
    }
}

fn property_types(node: &Value, types: &mut BTreeSet<String>) {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        types.insert(reference.trim_start_matches("#/$defs/Reply_").to_owned());
    } else if let Some(kind) = node.get("type") {
        for kind in std::iter::once(kind)
            .chain(kind.as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .filter(|kind| *kind != "null")
        {
            types.insert(format!(
                "{kind}:{}",
                node.get("format").unwrap_or(&Value::Null)
            ));
        }
    }
    for keyword in ["anyOf", "oneOf", "allOf"] {
        for branch in node
            .get(keyword)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            property_types(branch, types);
        }
    }
}

fn check(types: &Types) -> Result<(), String> {
    for (name, definitions) in types {
        if definitions.len() != 1 {
            return Err(format!(
                "temporal property {name} has conflicting types: {definitions:?}"
            ));
        }
    }
    Ok(())
}

pub(super) fn temporal_properties_have_one_type_across_registered_schemas() -> Result<(), String> {
    let mut types = Types::new();
    for contract in super::registry() {
        collect(&(contract.generate)(), &mut types);
    }
    check(&types)?;
    for (name, expected) in [
        ("updated", "PartialDate"),
        ("retrieved", "PartialDate"),
        ("source_updated", "SourceUpdated"),
        ("fetched", "integer:\"uint64\""),
    ] {
        if types.get(name) != Some(&BTreeSet::from([expected.into()])) {
            return Err(format!(
                "temporal property {name} must use {expected}, got {:?}",
                types.get(name)
            ));
        }
    }
    Ok(())
}

pub(super) fn temporal_property_check_rejects_conflicting_definition_types() -> Result<(), String> {
    let mut types = Types::new();
    collect(
        &json!({"properties":{"source_updated":{"$ref":"#/$defs/Reply_SourceUpdated"}},
        "$defs":{"Other":{"properties":{"source_updated":{"$ref":"#/$defs/Reply_Text"}}}}}),
        &mut types,
    );
    match check(&types) {
        Err(error) if error.contains("source_updated") => Ok(()),
        result => Err(format!(
            "planted temporal type conflict was not identified: {result:?}"
        )),
    }
}

#[test]
fn native_timestamp_string_schema_keeps_local_and_offset_profiles() {
    let schema = generate::<SourceDateTimeValue>();
    assert_eq!(schema["type"], "string");
    assert!(schema.get("anyOf").is_none());
    let validator = jsonschema::validator_for(&schema).unwrap();
    for (text, lexical, runtime) in [
        ("2024-05-03T10:00:00.000100+02:00", true, true),
        ("2020-05-08T17:33:59.756164", true, true),
        ("2020-05-08T17:33:59.756164Z", true, true),
        ("2024-02-30T10:00:00", true, false),
        ("2024-05-03T24:00:00", false, false),
        ("2024-05-03T10:00:00+24:00", false, false),
        ("2024-05-03T10:00:00.", false, false),
    ] {
        assert_eq!(
            validator.is_valid(&serde_json::json!(text)),
            lexical,
            "{text}"
        );
        assert_eq!(
            text.parse::<SourceDateTimeValue>().is_ok(),
            runtime,
            "{text}"
        );
    }
}
