//! Compact tool reply shapes selected from the canonical command schema catalog.
use serde_json::{Value, json};
use std::{collections::BTreeSet, io};

pub(super) fn for_tool(root: &crate::grammar::Root, name: &str) -> io::Result<Value> {
    let mut ids = crate::help::commands(root)?
        .into_iter()
        .filter(|command| match name {
            "search" => command.command.first().is_some_and(|word| word == "search"),
            "fetch" => command.command.first().is_some_and(|word| word == "get"),
            _ => true,
        })
        .flat_map(|command| command.reply_schemas)
        .collect::<BTreeSet<_>>();
    if name == "fetch" {
        ids.extend(crate::schema::local_replies(&["help".into()]));
    }
    let health = canonical(<biomcp_types::Health as biomcp_types::ResponseBody>::SCHEMA)?;
    let mut envelope = shape(&health)?;
    let properties = envelope
        .get_mut("properties")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| io::Error::other("missing envelope properties"))?;
    properties.insert("schema".into(), json!({"type":"string","enum":ids}));
    properties.insert("query".into(), json!({"type":["object","array","null"]}));
    properties.insert("result".into(), json!({"type":"object"}));
    let mut branches = vec![envelope, shape(&canonical("biomcp.error/4")?)?];
    if name == "biomcp" {
        branches.push(json!({"properties":{
            "$id":{"type":"string","pattern":"^(biomcp|biodata)\\.[a-z0-9_-]+/[1-9][0-9]*$"},
            "$schema":{"const":"https://json-schema.org/draft/2020-12/schema"}},
            "required":["$id","$schema"]}));
    } else {
        let id = match name {
            "search" => biomcp_types::ResearchSearch::<biomcp_types::ResearchEnvelope>::SCHEMA,
            "fetch" => biomcp_types::ResearchFetch::<biomcp_types::ResearchEnvelope>::SCHEMA,
            _ => return Err(io::Error::other("unknown research tool")),
        };
        branches.push(shape(&canonical(id)?)?);
    }
    Ok(json!({
        "$schema": super::protocol::SCHEMA_DIALECT,
        "type": "object",
        "anyOf": branches
    }))
}
fn canonical(id: &str) -> io::Result<Value> {
    serde_json::from_str(
        crate::schema::find(id).ok_or_else(|| io::Error::other("missing reply schema"))?,
    )
    .map_err(io::Error::other)
}
fn resolve<'a>(node: &'a Value, root: &'a Value) -> Option<&'a Value> {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        root.pointer(reference.strip_prefix('#')?)
    } else {
        Some(node)
    }
}
fn shape(root: &Value) -> io::Result<Value> {
    let properties = root
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| io::Error::other("missing reply properties"))?;
    let properties = properties
        .iter()
        .map(|(key, node)| {
            let node =
                resolve(node, root).ok_or_else(|| io::Error::other("unresolved reply property"))?;
            let value = if matches!(key.as_str(), "schema" | "status") {
                let mut compact = node.clone();
                if let Some(fields) = compact.as_object_mut() {
                    fields.remove("description");
                    fields.remove("title");
                }
                compact
            } else if let Some(kind) = node.get("type") {
                json!({"type":kind})
            } else if let Some(choices) = node.get("anyOf").and_then(Value::as_array) {
                let kinds = choices
                    .iter()
                    .filter_map(|choice| resolve(choice, root)?.get("type"))
                    .collect::<Vec<_>>();
                json!({"type":kinds})
            } else {
                return Err(io::Error::other("missing reply property type"));
            };
            Ok((key.clone(), value))
        })
        .collect::<io::Result<serde_json::Map<_, _>>>()?;
    Ok(json!({"properties":properties,"required":root.get("required")}))
}
