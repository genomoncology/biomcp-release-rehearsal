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
