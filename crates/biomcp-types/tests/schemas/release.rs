//! Check shared schema definitions and independently recorded release contracts.
use super::{compatibility, policy::read_reviews, registry};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

type Check<T> = Result<T, Box<dyn std::error::Error>>;

#[expect(
    clippy::disallowed_methods,
    reason = "The release contract test owns baseline file and environment I/O"
)]
pub(super) fn generated_schemas_preserve_release_contracts() -> Check<()> {
    let Some(baseline) = std::env::var_os("BIOMCP_SCHEMA_BASELINE") else {
        return Ok(());
    };
    let reviews = read_reviews()?;
    let mut old = read_baseline(Path::new(&baseline))?;
    for contract in registry() {
        let mut new = (contract.generate)();
        new.as_object_mut()
            .ok_or("generated schema is not an object")?
            .insert("$id".into(), contract.id.into());
        compatibility::check_reviewed(&new, &new, contract.kind, &reviews)?;
        if let Some(previous) = old.remove(contract.name) {
            compatibility::check_reviewed(&previous, &new, contract.kind, &reviews)
                .map_err(|error| format!("{}: {error}", contract.name))?;
        }
    }
    if !old.is_empty() {
        return Err(format!("removed released schemas: {:?}", old.keys()).into());
    }
    Ok(())
}

#[expect(
    clippy::disallowed_methods,
    reason = "Read the independently supplied schema snapshot at the release contract boundary"
)]
fn read_baseline(root: &Path) -> Check<BTreeMap<String, Value>> {
    let mut schemas = BTreeMap::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            return Err(format!("unexpected baseline file: {}", path.display()).into());
        }
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or("invalid baseline filename")?
            .to_owned();
        let value = serde_json::from_slice(&fs::read(path)?)?;
        compatibility::check(&value, &value)?;
        schemas.insert(name, value);
    }
    if schemas.is_empty() {
        return Err("empty schema baseline".into());
    }
    Ok(schemas)
}

#[expect(
    clippy::disallowed_methods,
    reason = "Schema definition contract tests own file I/O outside the pure library"
)]
pub(super) fn checked_in_schema_definition_names_have_one_body() -> Check<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let mut definitions = BTreeMap::new();
    for contract in registry() {
        let path = root.join(format!("{}.json", contract.name));
        let schema: Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
        for (name, body) in schema
            .get("$defs")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            assert_eq!(
                name.starts_with("Reply_"),
                matches!(contract.kind, super::policy::ContractKind::Reply),
                "definition {name} has the wrong namespace in {}",
                contract.name
            );
            if let Some((previous, owner)) = definitions.get(name) {
                assert_eq!(
                    previous, body,
                    "definition {name} differs between {owner} and {}",
                    contract.name
                );
            } else {
                definitions.insert(name.clone(), (body.clone(), contract.name));
            }
        }
    }
    Ok(())
}

// x-identifier marks authority/value objects, including numeric values, wherever a schema nests them.
// Entity records and relations with already qualified scalar IDs are not identifier objects.
// Native reference records retain category fields and optional reference IDs; a category is not an authority.
pub(super) fn identifier_objects_use_namespace_and_id() -> Check<()> {
    for contract in registry() {
        identifier_shape(&(contract.generate)(), contract.name)?;
    }
    let numeric = serde_json::json!({
        "x-identifier":true, "properties":{"namespace":{"type":"string"},"id":{"type":"integer"}},
        "required":["namespace","id"]
    });
    identifier_shape(
        &serde_json::json!({"$defs":{"Nested":numeric.clone()}}),
        "numeric",
    )?;
    for field in ["namespace", "id"] {
        let mut malformed = numeric.clone();
        *malformed
            .get_mut("required")
            .ok_or("numeric required names")? = serde_json::json!([field]);
        assert!(
            identifier_shape(
                &serde_json::json!({"$defs":{"Nested":malformed}}),
                "numeric"
            )
            .is_err(),
            "identifier with only {field} required must be rejected"
        );
    }
    Ok(())
}

fn identifier_shape(node: &Value, owner: &str) -> Check<()> {
    match node {
        Value::Object(object) => {
            if object.get("x-identifier") == Some(&Value::Bool(true)) {
                let branches = object.get("oneOf").and_then(Value::as_array);
                for branch in branches.map_or_else(|| vec![node], |items| items.iter().collect()) {
                    let properties = branch["properties"]
                        .as_object()
                        .ok_or("identifier properties")?;
                    let required = branch["required"]
                        .as_array()
                        .ok_or("identifier required names")?;
                    for field in ["namespace", "id"] {
                        if !properties.contains_key(field) || !required.iter().any(|v| v == field) {
                            return Err(format!("{owner}: identifier requires {field}").into());
                        }
                    }
                    for old in ["source", "kind", "value", "database", "accession"] {
                        if properties.contains_key(old) {
                            return Err(format!("{owner}: identifier still uses {old}").into());
                        }
                    }
                }
            }
            for child in object.values() {
                identifier_shape(child, owner)?;
            }
        }
        Value::Array(items) => {
            for child in items {
                identifier_shape(child, owner)?;
            }
        }
        _ => {}
    }
    Ok(())
}
