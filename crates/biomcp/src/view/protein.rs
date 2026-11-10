//! Coordinate guidance shared by human and MCP text; numeric spans stay typed JSON.
use biomcp_types::ResponseBody;
use serde_json::Value;

pub(crate) fn note(value: &Value) -> Option<String> {
    let identity = value.get("schema")?.as_str()?;
    if ![
        biomcp_types::ProteinFeatures::SCHEMA,
        biomcp_types::ProteinDomains::SCHEMA,
        biomcp_types::ProteinCard::SCHEMA,
    ]
    .contains(&identity)
        || !has_span(value.get("result")?)
    {
        return None;
    }
    let schema: Value = serde_json::from_str(crate::schema::find(identity)?).ok()?;
    let description = schema
        .pointer("/$defs/Reply_UncertainSpan0/description")?
        .as_str()?;
    Some(format!(
        "Coordinates: {description} Exact spans are half-open [start, end): start is included and end is excluded. \
         Original UniProt/InterPro source coordinates are one-based inclusive; source_coordinates describes that original convention. \
         The --position selector remains one-based."
    ))
}
fn has_span(value: &Value) -> bool {
    match value {
        Value::Object(fields) => fields.contains_key("span") || fields.values().any(has_span),
        Value::Array(items) => items.iter().any(has_span),
        _ => false,
    }
}

pub(crate) fn append_mcp(result: &mut Value, value: &Value) {
    if let Some(note) = note(value)
        && let Some(content) = result.get_mut("content").and_then(Value::as_array_mut)
    {
        content.push(serde_json::json!({"type":"text","text":note}));
    }
}
