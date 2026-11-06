//! Project only protocol-owned Steps; upstream text and query values remain data.
use serde_json::Value;

pub(super) fn project(
    root: &crate::grammar::Root,
    value: &mut Value,
    tools: &super::grouping::Catalog,
) -> std::io::Result<Vec<Value>> {
    let Some(id) = value.get("schema").and_then(Value::as_str) else {
        return Ok(Vec::new());
    };
    let text =
        crate::schema::find(id).ok_or_else(|| std::io::Error::other("missing reply schema"))?;
    let schema: Value = serde_json::from_str(text).map_err(std::io::Error::other)?;
    let mut calls = Vec::new();
    for path in crate::schema::steps::paths(&schema, &schema, value)
        .into_iter()
        .rev()
    {
        let call = value.pointer(&path).and_then(|step| {
            let args = without_globals(root, step.get("args")?.as_array()?)?;
            let mut call = super::summary::translate(root, &args, tools)?;
            if let Some(kind) = step.get("kind") {
                call.as_object_mut()?.insert("kind".into(), kind.clone());
            }
            Some(call)
        });
        if let Some(call) = call {
            if let Some(step) = value.pointer_mut(&path) {
                *step = call.clone();
            }
            calls.push(call);
        } else {
            crate::schema::steps::remove(value, &path);
        }
    }
    calls.reverse();
    Ok(calls)
}

fn without_globals(root: &crate::grammar::Root, args: &[Value]) -> Option<Vec<Value>> {
    let mut result = Vec::new();
    let mut words = args.iter();
    while let Some(word) = words.next() {
        let text = word.as_str()?;
        if text == "--" {
            result.push(word.clone());
            result.extend(words.cloned());
            break;
        }
        let flag = text
            .strip_prefix("--")
            .unwrap_or_default()
            .split('=')
            .next()?;
        // Typed tools do not advertise the force-refresh global. Omitting it would
        // change a refresh into an ordinary cached call, so omit that suggestion.
        if flag == "fresh" {
            return None;
        }
        if let Some(global) = root
            .get_arguments()
            .find(|arg| arg.get_long() == Some(flag))
        {
            if global.get_action().takes_values() && !text.contains('=') {
                words.next()?;
            }
        } else {
            result.push(word.clone());
        }
    }
    Some(result)
}

pub(super) fn allowed(call: &Value, tools: &[Value]) -> bool {
    tools
        .iter()
        .find(|tool| tool.get("name") == call.get("name"))
        .is_some_and(|tool| choices(&tool["inputSchema"], &call["arguments"]))
}

// The canonical parser already checks types, ranges and required fields. The
// advertised schema can additionally remove fields and narrow enum choices.
fn choices(schema: &Value, value: &Value) -> bool {
    if schema
        .get("enum")
        .and_then(Value::as_array)
        .is_some_and(|values| !values.contains(value))
    {
        return false;
    }
    if let Some(branches) = schema
        .get("anyOf")
        .or_else(|| schema.get("oneOf"))
        .and_then(Value::as_array)
    {
        return branches.iter().any(|branch| choices(branch, value));
    }
    if schema.get("type") == Some(&Value::String("null".into())) {
        return value.is_null();
    }
    match value {
        Value::Object(fields) => fields.iter().all(|(key, value)| {
            schema
                .get("properties")
                .and_then(|fields| fields.get(key))
                .map_or_else(
                    || schema.get("additionalProperties") != Some(&Value::Bool(false)),
                    |node| choices(node, value),
                )
        }),
        Value::Array(values) => schema
            .get("items")
            .is_none_or(|node| values.iter().all(|value| choices(node, value))),
        _ => true,
    }
}
