//! Publish input constraints separately from extensible output descriptions.
use super::{generate, policy::ContractKind};
use schemars::JsonSchema;
use serde_json::Value;

pub(super) fn reply<T: JsonSchema>() -> Value {
    let mut schema = generate::<T>();
    prepare(&mut schema, ContractKind::Reply);
    schema
}

pub(super) fn query<T: JsonSchema>() -> Value {
    let mut schema = generate::<T>();
    prepare(&mut schema, ContractKind::Query);
    schema
}

fn prepare(node: &mut Value, kind: ContractKind) {
    match node {
        Value::Object(object) => {
            let growable = object.remove("x-enum-open") == Some(Value::Bool(true));
            if matches!(kind, ContractKind::Reply) {
                if let Some(Value::Object(definitions)) = object.get_mut("$defs") {
                    *definitions = std::mem::take(definitions)
                        .into_iter()
                        .map(|(name, body)| (format!("Reply_{name}"), body))
                        .collect();
                }
                if let Some(Value::String(reference)) = object.get_mut("$ref")
                    && let Some(name) = reference.strip_prefix("#/$defs/")
                {
                    *reference = format!("#/$defs/Reply_{name}");
                }
                if object.get("additionalProperties") == Some(&Value::Bool(false)) {
                    object.remove("additionalProperties");
                }
                if growable && let Some(known) = object.remove("enum") {
                    object.insert("x-extensible-enum".into(), known);
                }
            }
            for child in object.values_mut() {
                prepare(child, kind);
            }
        }
        Value::Array(children) => {
            for child in children {
                prepare(child, kind);
            }
        }
        _ => {}
    }
}

/// Publish the same choice documentation for CLI help, schema readers and MCP discovery.
pub(super) fn command_descriptions(schema: &mut Value, id: &str) {
    let Some(command) = biomcp_types::COMMANDS.iter().find(|row| row.schema == id) else {
        return;
    };
    let root = schema.clone();
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        for (name, property) in properties {
            let original = property
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if name == "section"
                && let Some(object) = property.as_object_mut()
            {
                object.insert("x-section-rule".into(), original.clone().into());
            }
            let form = choice_form(property, &root);
            if form.is_none() && name != "section" {
                continue;
            }
            let first = (name == "section")
                .then(|| property.get("title").and_then(Value::as_str))
                .flatten()
                .unwrap_or_else(|| original.lines().next().unwrap_or_default());
            if first.contains("biomcp_help") {
                continue;
            }
            let suffix = if form.is_some() {
                format!("; biomcp_help {} {}", command.verb, command.entity)
            } else {
                String::new()
            };
            let wording = if first.is_empty() {
                format!("{} {}", name.replace('_', " "), form.unwrap_or("text"))
            } else {
                first.into()
            };
            let prefix: String = wording
                .chars()
                .take(80_usize.saturating_sub(suffix.len()))
                .collect();
            let prefix = if prefix.len() < wording.len() {
                prefix
                    .rsplit_once(' ')
                    .map_or(prefix.as_str(), |(words, _)| words)
            } else {
                prefix.as_str()
            };
            let summary = if suffix.is_empty() {
                prefix.to_owned()
            } else {
                format!("{}{suffix}", prefix.trim_end_matches('.'))
            };
            let description = if name == "section" {
                summary
            } else {
                format!("{summary}\n{original}")
            };
            if let Some(object) = property.as_object_mut() {
                object.insert("description".into(), description.into());
            }
        }
    }
}
fn choice_form(node: &Value, root: &Value) -> Option<&'static str> {
    if let Some(values) = node.get("enum").and_then(Value::as_array) {
        return values
            .first()
            .map(|value| if value.is_string() { "text" } else { "value" });
    }
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        return choice_form(root.pointer(reference.strip_prefix('#')?)?, root);
    }
    if let Some(items) = node.get("items") {
        return choice_form(items, root);
    }
    ["anyOf", "oneOf"].into_iter().find_map(|key| {
        node.get(key)
            .and_then(Value::as_array)?
            .iter()
            .find_map(|branch| choice_form(branch, root))
    })
}

pub(super) fn section_rules_keep_the_complete_source_owned_prose() -> Result<(), String> {
    for contract in super::commands::registry() {
        if !biomcp_types::COMMANDS
            .iter()
            .any(|row| row.schema == contract.id)
        {
            continue;
        }
        let mut schema = (contract.generate)();
        let Some(original) = schema.pointer("/properties/section/description").cloned() else {
            continue;
        };
        command_descriptions(&mut schema, contract.id);
        if schema.pointer("/properties/section/x-section-rule") != Some(&original) {
            return Err(format!("{}: lost full section rule", contract.id));
        }
        let description = schema
            .pointer("/properties/section/description")
            .and_then(Value::as_str)
            .ok_or("missing compact section description")?;
        if description.contains('\n') || description.chars().count() > 80 {
            return Err(format!(
                "{}: invalid compact section description",
                contract.id
            ));
        }
    }
    Ok(())
}
