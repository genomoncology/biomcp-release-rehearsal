//! Only the canonical response schema can identify executable Step fields.
use serde_json::Value;
use std::collections::BTreeSet;

const STEP: &str = "#/$defs/Reply_Step";

pub(crate) fn paths(schema: &Value, node: &Value, value: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    visit(schema, node, value, "", &mut Vec::new(), &mut paths);
    let mut seen = BTreeSet::new();
    paths.retain(|path| seen.insert(path.clone()));
    paths
}
fn visit<'a>(
    root: &'a Value,
    node: &'a Value,
    value: &Value,
    path: &str,
    references: &mut Vec<&'a Value>,
    paths: &mut Vec<String>,
) {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        if reference == STEP {
            if value.is_object() {
                paths.push(path.to_owned());
            }
            return;
        }
        if let Some(target) = reference
            .strip_prefix('#')
            .and_then(|path| root.pointer(path))
            && !references
                .iter()
                .any(|previous| std::ptr::eq(*previous, target))
        {
            references.push(target);
            visit(root, target, value, path, references, paths);
            references.pop();
        }
        return;
    }
    if !matches(node, value) {
        return;
    }
    for key in ["anyOf", "oneOf", "allOf"] {
        for branch in node
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            visit(root, branch, value, path, references, paths);
        }
    }
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if let Some(property) = node
                    .get("properties")
                    .and_then(|properties| properties.get(key))
                {
                    let key = key.replace('~', "~0").replace('/', "~1");
                    visit(
                        root,
                        property,
                        value,
                        &format!("{path}/{key}"),
                        &mut Vec::new(),
                        paths,
                    );
                }
            }
        }
        Value::Array(values) => {
            if let Some(items) = node.get("items") {
                for (index, value) in values.iter().enumerate() {
                    visit(
                        root,
                        items,
                        value,
                        &format!("{path}/{index}"),
                        &mut Vec::new(),
                        paths,
                    );
                }
            }
        }
        _ => {}
    }
}
fn matches(schema: &Value, value: &Value) -> bool {
    let kind = |name: &str| match name {
        "null" => value.is_null(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        _ => true,
    };
    if let Some(kinds) = schema.get("type") {
        let valid = kinds.as_str().map_or_else(
            || {
                kinds
                    .as_array()
                    .is_none_or(|kinds| kinds.iter().filter_map(Value::as_str).any(kind))
            },
            kind,
        );
        if !valid {
            return false;
        }
    }
    if !literal(schema, value) {
        return false;
    }
    if let Some(fields) = value.as_object() {
        if schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|required| {
                required
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|key| !fields.contains_key(key))
            })
        {
            return false;
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            return properties.iter().all(|(key, property)| {
                fields.get(key).is_none_or(|value| literal(property, value))
            });
        }
    }
    true
}
fn literal(schema: &Value, value: &Value) -> bool {
    schema.get("const").is_none_or(|expected| expected == value)
        && schema
            .get("enum")
            .and_then(Value::as_array)
            .is_none_or(|values| values.contains(value))
}

pub(crate) fn remove(value: &mut Value, path: &str) {
    let Some((parent, key)) = path.rsplit_once('/') else {
        return;
    };
    let key = key.replace("~1", "/").replace("~0", "~");
    match value.pointer_mut(parent) {
        Some(Value::Object(fields)) => {
            fields.remove(&key);
        }
        Some(Value::Array(items)) => {
            if let Ok(index) = key.parse::<usize>()
                && index < items.len()
            {
                items.remove(index);
            }
        }
        _ => {}
    }
}
