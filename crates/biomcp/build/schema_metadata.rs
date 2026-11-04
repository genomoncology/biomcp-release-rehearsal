//! Derive discovery metadata from the same artifacts embedded by the build.
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

pub(super) fn write(
    schemas: &BTreeMap<String, String>,
    output: &Path,
) -> Result<(), Box<dyn Error>> {
    let parsed = schemas
        .iter()
        .map(|(id, text)| serde_json::from_str::<Value>(text).map(|value| (id, value)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut catalog = String::from("&[\n");
    let mut replies = String::from("&[\n");
    for (id, schema) in &parsed {
        if (id.starts_with("biomcp.") || id.starts_with("biodata.")) && id.contains('_') {
            return Err("schema identities must use hyphens".into());
        }
        let result = schema
            .pointer("/properties/result/anyOf/0/$ref")
            .and_then(Value::as_str)
            .and_then(|reference| schema.pointer(reference.trim_start_matches('#')));
        let description = result
            .and_then(|body| body["description"].as_str())
            .or_else(|| schema["description"].as_str())
            .or_else(|| schema["title"].as_str())
            .ok_or("schema requires a description or title")?;
        let description = description.split_whitespace().collect::<Vec<_>>().join(" ");
        catalog.push_str(&format!("({id:?}, {description:?}),\n"));
        let Some(title) = schema["title"]
            .as_str()
            .filter(|title| *title != "Envelope")
        else {
            continue;
        };
        let mut titles = BTreeSet::from([title.trim_end_matches("Command").to_owned()]);
        if let Some(aliases) = schema["x-reply-query-titles"].as_array() {
            titles.extend(aliases.iter().filter_map(Value::as_str).map(str::to_owned));
        }
        let matches = parsed
            .iter()
            .filter_map(|(reply_id, reply)| {
                reply.pointer("/properties/schema/const")?;
                let query = reply.pointer("/properties/query")?;
                let mut names = BTreeSet::new();
                references(query, reply, &mut names);
                (!titles.is_disjoint(&names)).then_some(format!("{reply_id:?}"))
            })
            .collect::<Vec<_>>()
            .join(",");
        replies.push_str(&format!("({id:?}, &[{matches}]),\n"));
    }
    fs::write(output.join("schema-catalog.rs"), format!("{catalog}]"))?;
    fs::write(output.join("schema-replies.rs"), format!("{replies}]"))?;
    Ok(())
}
fn references(node: &Value, root: &Value, names: &mut BTreeSet<String>) {
    match node {
        Value::Object(fields) => {
            if let Some(reference) = fields.get("$ref").and_then(Value::as_str)
                && let Some(name) = reference.strip_prefix("#/$defs/")
                && names.insert(name.strip_prefix("Reply_").unwrap_or(name).into())
                && let Some(target) = root.pointer(reference.trim_start_matches('#'))
            {
                references(target, root, names);
            }
            for (key, value) in fields {
                if key != "$ref" {
                    references(value, root, names);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                references(value, root, names);
            }
        }
        _ => {}
    }
}
