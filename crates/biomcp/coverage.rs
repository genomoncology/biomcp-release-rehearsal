//! Source selector discovery shared by build-time and runtime help.
/// Declared source selectors plus implicit sources used by a command and its sections.
pub(crate) fn command_sources(
    verb: &str,
    entity: &str,
    schema: &serde_json::Value,
) -> std::collections::BTreeSet<String> {
    let mut sources = std::collections::BTreeSet::new();
    for key in ["source", "sources", "resolver"] {
        if let Some(node) = schema.pointer(&format!("/properties/{key}")) {
            source_words(node, schema, &mut sources);
        }
    }
    sources.extend(
        biomcp_types::command_implicit_sources(verb, entity)
            .iter()
            .map(ToString::to_string),
    );
    sources
}

pub(crate) fn source_words(
    node: &serde_json::Value,
    schema: &serde_json::Value,
    sources: &mut std::collections::BTreeSet<String>,
) {
    if let Some(reference) = node.get("$ref").and_then(serde_json::Value::as_str)
        && let Some(target) = schema.pointer(reference.trim_start_matches('#'))
    {
        source_words(target, schema, sources);
    }
    for key in ["enum", "x-extensible-enum"] {
        for value in node
            .get(key)
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_str)
        {
            if let Ok(source) = value.parse::<biomcp_types::Source>() {
                sources.insert(source.to_string());
            }
        }
    }
    if let Some(items) = node.get("items") {
        source_words(items, schema, sources);
    }
    for key in ["anyOf", "oneOf"] {
        for child in node
            .get(key)
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            source_words(child, schema, sources);
        }
    }
}
