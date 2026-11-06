//! Compact public schemas and checked argument-to-CLI spelling.
use serde_json::{Value, json};
use std::ffi::OsString;

pub(super) fn resolve<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
    node.get("$ref")
        .and_then(Value::as_str)
        .and_then(|reference| root.pointer(reference.trim_start_matches('#')))
        .unwrap_or(node)
}
fn compact(node: &Value, root: &Value, optional: bool, enum_limit: usize) -> Value {
    let node = resolve(node, root);
    let Some(fields) = node.as_object() else {
        return node.clone();
    };
    let mut result = serde_json::Map::new();
    for (key, value) in fields {
        match key.as_str() {
            "type" if optional && value.is_array() => {
                let kinds: Vec<_> = value
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|kind| *kind != "null")
                    .cloned()
                    .collect();
                result.insert(
                    key.clone(),
                    if kinds.len() == 1 {
                        kinds.first().cloned().unwrap_or(Value::Null)
                    } else {
                        json!(kinds)
                    },
                );
            }
            "enum" if value.as_array().is_some_and(|v| v.len() > enum_limit) => {
                result.insert("type".into(), json!("string"));
            }
            "type"
            | "enum"
            | "const"
            | "minimum"
            | "maximum"
            | "minItems"
            | "maxItems"
            | "required"
            | "additionalProperties" => {
                result.insert(key.clone(), value.clone());
            }
            "items" => {
                result.insert(key.clone(), compact(value, root, false, enum_limit));
            }
            "anyOf" | "oneOf" => {
                let branches: Vec<_> = value
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|v| !optional || resolve(v, root).get("type") != Some(&json!("null")))
                    .map(|v| compact(v, root, optional, enum_limit))
                    .collect();
                if branches.len() == 1 {
                    if let Some(fields) = branches.first().and_then(Value::as_object) {
                        result.extend(fields.clone());
                    }
                } else {
                    result.insert(key.clone(), json!(branches));
                }
            }
            _ => {}
        }
    }
    Value::Object(result)
}
pub(super) fn input(schema: &Value) -> Value {
    input_with_limit(schema, crate::config::DEFAULT_TYPED_ENUM_LIMIT)
}
pub(super) fn input_with_limit(schema: &Value, enum_limit: usize) -> Value {
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .flat_map(|(key, node)| {
            let description = node
                .get("x-section-rule")
                .or_else(|| node.get("description"))
                .and_then(Value::as_str)
                .map(|text| {
                    let mut lines = text.lines();
                    let first = lines.next().unwrap_or_default();
                    // Generated schemas prepend a compact help pointer to the original prose.
                    let text = if first.contains("biomcp_help") {
                        lines
                            .next()
                            .unwrap_or(first)
                            .split("; biomcp_help")
                            .next()
                            .unwrap_or(first)
                            .to_owned()
                    } else {
                        first.to_owned()
                    };
                    text.split(". ")
                        .next()
                        .unwrap_or_default()
                        .trim_end_matches('.')
                        .to_owned()
                        + "."
                });
            if let Some(flags) = node.get("x-cli-flags").and_then(Value::as_object) {
                flags
                    .keys()
                    .map(|flag| {
                        let mut property = json!({"type":"boolean"});
                        if let Some(text) = &description
                            && let Some(fields) = property.as_object_mut()
                        {
                            fields.insert("description".into(), json!(text));
                        }
                        (flag.clone(), property)
                    })
                    .collect()
            } else {
                let optional = !schema
                    .get("required")
                    .and_then(Value::as_array)
                    .is_some_and(|v| v.contains(&json!(key)));
                let mut property = compact(node, schema, optional, enum_limit);
                if let Some(text) = description
                    && let Some(fields) = property.as_object_mut()
                {
                    fields.insert("description".into(), json!(text));
                }
                vec![(key.clone(), property)]
            }
        })
        .collect::<serde_json::Map<_, _>>();
    let mut result = json!({"$schema":super::super::protocol::SCHEMA_DIALECT,
        "type":"object", "properties":properties,"additionalProperties":false});
    if let Some(required) = schema.get("required")
        && let Some(fields) = result.as_object_mut()
    {
        fields.insert("required".into(), required.clone());
    }
    result
}
fn words(value: &Value) -> Vec<String> {
    match value {
        Value::String(value) => vec![value.clone()],
        Value::Array(values) => values.iter().flat_map(words).collect(),
        _ => vec![value.to_string()],
    }
}
pub(super) fn cli(
    root: &crate::grammar::Root,
    verb: &str,
    entity: &str,
    schema: &Value,
    values: &Value,
) -> Option<Vec<OsString>> {
    let command = root.find_subcommand(verb)?.find_subcommand(entity)?;
    let mut args = vec![verb.into(), entity.into()];
    // Parser indices own positional order; flags precede them to keep literal values safe.
    let mut positional = Vec::new();
    for (key, value) in values.as_object()? {
        let argument = command
            .get_arguments()
            .find(|arg| arg.get_id().as_str() == key)?;
        if let Some(index) = argument.get_index() {
            positional.push((index, words(value)));
            continue;
        }
        let flag = argument.get_long()?;
        if let Some(boolean) = value.as_bool()
            && matches!(
                argument.get_action(),
                clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
            )
        {
            if argument.is_require_equals_set() {
                args.push(format!("--{flag}={boolean}").into());
            } else if boolean {
                args.push(format!("--{flag}").into());
            }
            continue;
        }
        let values = words(value);
        if schema
            .get("properties")
            .and_then(|fields| fields.get(key))
            .and_then(|field| field.get("x-comma-separated"))
            == Some(&Value::Bool(true))
        {
            args.push(format!("--{flag}={}", values.join(",")).into());
        } else {
            for value in values {
                args.push(format!("--{flag}={value}").into());
            }
        }
    }
    positional.sort_by_key(|(index, _)| *index);
    if positional
        .iter()
        .flat_map(|(_, words)| words)
        .any(|word| word.starts_with('-'))
    {
        args.push("--".into());
    }
    args.extend(
        positional
            .into_iter()
            .flat_map(|(_, values)| values)
            .map(OsString::from),
    );
    Some(args)
}
