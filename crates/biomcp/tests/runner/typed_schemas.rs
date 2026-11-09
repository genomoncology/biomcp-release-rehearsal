//! Installed tool catalogs keep command argument documentation and fit discovery budgets.
use crate::runner::{
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};
use std::io::Write;

fn field<'a>(node: &'a Value, key: &str) -> &'a Value {
    node.get(key).unwrap_or(&Value::Null)
}

pub(super) fn check(env: &Environment) -> Result<(), String> {
    let tools = catalog(env, json!({"mcp_tool_mode":"typed"}))?;
    let views = crate::runner::protocol::typed_catalog::commands(&tools);
    let mut failures = Vec::new();
    top_level(&tools, &mut failures);
    for (verb, entity, id) in biomcp_types::command_paths() {
        let name = format!("{verb}_{entity}").replace('-', "_");
        let tool = views
            .iter()
            .find(|tool| tool["name"] == name)
            .ok_or_else(|| format!("{name}: no advertised action"))?;
        if field(tool, "description")
            .as_str()
            .is_some_and(|s| s.contains('\n'))
        {
            failures.push(format!(
                "{name}: full schema prose leaked into action summary"
            ));
        }
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &["schema".into(), id.into()])
            .output()
            .map_err(|e| e.to_string())?;
        let schema: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        for (key, node) in schema
            .get("properties")
            .and_then(Value::as_object)
            .ok_or("properties")?
        {
            if node.get("x-cli-flags").is_some() {
                continue;
            }
            let property = tool
                .get("inputSchema")
                .and_then(|s| s.get("properties"))
                .and_then(|p| p.get(key))
                .ok_or("property")?;
            let expected = description(node);
            if property.get("description").and_then(Value::as_str) != expected.as_deref() {
                failures.push(format!("{name}.{key}: lost own description"));
            }
            if nested_description(property, true) {
                failures.push(format!("{name}.{key}: nested description"));
            }
            let required = schema
                .get("required")
                .and_then(Value::as_array)
                .is_some_and(|v| v.contains(&json!(key)));
            if !required && nullable(property) {
                failures.push(format!("{name}.{key}: optional nullable branch"));
            }
            if oversized_enum(property, 30) {
                failures.push(format!("{name}.{key}: oversized enum"));
            }
        }
    }
    report(&tools)?;
    // Exercise the same budget rejection used for the installed catalog.
    if budget(&json!({"name":"oversized", "description":"x".repeat(2001), "inputSchema":{}}))
        .is_ok()
    {
        return Err("oversized synthetic tool passed budget".into());
    }
    for tool in &views {
        if let Err(error) = budget(tool) {
            failures.push(error);
        }
    }
    if !failures.is_empty() {
        return Err(failures.join("\n"));
    }
    super::typed_catalog::budgets(env, &tools)?;
    settings(env, &tools)
}
/// The entity tools themselves: one-line descriptions and no default output schema.
fn top_level(tools: &[Value], failures: &mut Vec<String>) {
    for tool in tools {
        let name = field(tool, "name");
        if ["biomcp", "search", "fetch"].contains(&name.as_str().unwrap_or_default()) {
            continue; // Protocol tools keep the output schemas they had before.
        }
        if field(tool, "description")
            .as_str()
            .is_some_and(|s| s.contains('\n'))
        {
            failures.push(format!(
                "{name}: full schema prose leaked into tool summary"
            ));
        }
        if tool.get("outputSchema").is_some() {
            failures.push(format!("{name}: outputSchema must default off"));
        }
    }
}
fn catalog(env: &Environment, config: Value) -> Result<Vec<Value>, String> {
    env.config(&config).map_err(|e| e.to_string())?;
    let replies = exchange_with_globals(
        env,
        &[request(json!(1), "tools/list", json!({}))],
        &["--offline".into()],
    )?;
    let tools = replies
        .first()
        .and_then(|r| r.pointer("/result/tools"))
        .and_then(Value::as_array)
        .cloned()
        .ok_or("catalog missing")?;
    super::typed_catalog::check(&tools)?;
    Ok(tools)
}
fn nested_description(node: &Value, top: bool) -> bool {
    match node {
        Value::Object(map) => {
            (!top && map.contains_key("description"))
                || map
                    .iter()
                    .any(|(k, v)| k != "description" && nested_description(v, false))
        }
        Value::Array(nodes) => nodes.iter().any(|n| nested_description(n, false)),
        _ => false,
    }
}
fn nullable(node: &Value) -> bool {
    match node {
        Value::Object(map) => {
            map.get("type").is_some_and(|v| {
                v == "null" || v.as_array().is_some_and(|a| a.contains(&json!("null")))
            }) || map.values().any(nullable)
        }
        Value::Array(nodes) => nodes.iter().any(nullable),
        _ => false,
    }
}
fn oversized_enum(node: &Value, limit: usize) -> bool {
    match node {
        Value::Object(map) => {
            map.get("enum")
                .and_then(Value::as_array)
                .is_some_and(|v| v.len() > limit)
                || map.values().any(|v| oversized_enum(v, limit))
        }
        Value::Array(nodes) => nodes.iter().any(|n| oversized_enum(n, limit)),
        _ => false,
    }
}
pub(super) fn size(tool: &Value) -> usize {
    ["name", "description"]
        .iter()
        .map(|key| {
            field(tool, key)
                .as_str()
                .unwrap_or_default()
                .chars()
                .count()
        })
        .sum::<usize>()
        + field(tool, "inputSchema").to_string().chars().count()
}
pub(super) fn budget(tool: &Value) -> Result<(), String> {
    let length = size(tool);
    // search_trial exposes the complete supported filter set. Even with no descriptions,
    // its supported filter schema structure is 2058 characters; these optional filters cannot fit
    // the ordinary budget. Restore source-owned numeric guidance within the sole PM-approved exception of 3500.
    let limit = if field(tool, "name") == "search_trial" {
        3500
    } else {
        2000
    };
    if length > limit {
        Err(format!(
            "{}: {length} characters exceeds {limit}",
            field(tool, "name")
        ))
    } else {
        Ok(())
    }
}
fn report(tools: &[Value]) -> Result<(), String> {
    let mut output = std::io::stdout().lock();
    for tool in tools {
        writeln!(
            output,
            "TYPED_TOOL_SIZE {} characters={} output_schema_characters={}",
            field(tool, "name"),
            size(tool),
            tool.get("outputSchema")
                .map_or(0, |v| v.to_string().chars().count())
        )
        .map_err(|e| e.to_string())?;
    }
    writeln!(
        output,
        "TYPED_TOOL_LIST characters={} tools={} budget_characters={}",
        json!(tools).to_string().chars().count(),
        tools.len(),
        tools.iter().map(size).sum::<usize>()
    )
    .map_err(|e| e.to_string())
}
fn settings(env: &Environment, default: &[Value]) -> Result<(), String> {
    let enabled = catalog(
        env,
        json!({"mcp_tool_mode":"typed", "mcp_typed_output_schema":true,"mcp_typed_definition_bytes":200000}),
    )?;
    report(&enabled)?;
    for tool in enabled {
        for (name, id) in [
            (
                "disease",
                <biomcp_types::DiseaseSites as biomcp_types::ResponseBody>::SCHEMA,
            ),
            (
                "cell_line",
                <biomcp_types::CellLinePage as biomcp_types::ResponseBody>::SCHEMA,
            ),
            (
                "cell_line",
                <biomcp_types::CellLineFilteredPage as biomcp_types::ResponseBody>::SCHEMA,
            ),
        ] {
            if field(&tool, "name").as_str() == Some(name)
                && !tool
                    .pointer("/outputSchema/anyOf/0/properties/schema/enum")
                    .and_then(Value::as_array)
                    .is_some_and(|ids| ids.contains(&json!(id)))
            {
                return Err(format!("{name} output schema omits {id}"));
            }
        }
        if matches!(
            field(&tool, "name").as_str(),
            Some("help" | "biomcp" | "search" | "fetch")
        ) {
            if !default.contains(&tool) {
                return Err("shared help/research contract changed".into());
            }
            continue;
        }
        if tool
            .pointer("/outputSchema/anyOf/0/properties/schema/enum")
            .and_then(Value::as_array)
            .is_none()
            || tool.pointer("/outputSchema/anyOf/1/properties").is_none()
        {
            return Err(format!(
                "{}: output schema not restored",
                field(&tool, "name")
            ));
        }
        let mut without = tool.clone();
        without
            .as_object_mut()
            .ok_or("tool object")?
            .remove("outputSchema");
        if !default.contains(&without) {
            return Err("output schema setting changed input catalog".into());
        }
    }
    enum_settings(env, default)
}
fn enum_settings(env: &Environment, default: &[Value]) -> Result<(), String> {
    let large = crate::runner::protocol::typed_catalog::commands(&catalog(
        env,
        json!({"mcp_tool_mode":"typed", "mcp_typed_enum_limit":1000}),
    )?);
    let default = crate::runner::protocol::typed_catalog::commands(default);
    if !large
        .iter()
        .any(|tool| oversized_enum(field(tool, "inputSchema"), 30))
    {
        return Err("raised enum limit did not restore large choices".into());
    }
    if !default
        .iter()
        .any(|tool| oversized_enum(field(tool, "inputSchema"), 1))
    {
        return Err("short enums lost their choices".into());
    }
    let small = crate::runner::protocol::typed_catalog::commands(&catalog(
        env,
        json!({"mcp_tool_mode":"typed", "mcp_typed_enum_limit":1}),
    )?);
    if small
        .iter()
        .any(|tool| oversized_enum(field(tool, "inputSchema"), 1))
    {
        return Err("enum limit setting ignored".into());
    }
    for tool in &large {
        let name = field(tool, "name").as_str().ok_or("tool name")?;
        let compact = small
            .iter()
            .find(|t| field(t, "name").as_str() == Some(name))
            .ok_or("lowered tool")?;
        for (key, property) in field(field(tool, "inputSchema"), "properties")
            .as_object()
            .ok_or("properties")?
        {
            if oversized_enum(property, 1) {
                let description = field(field(field(compact, "inputSchema"), "properties"), key)
                    .get("description")
                    .and_then(Value::as_str)
                    .ok_or("choice description")?;
                if description.is_empty()
                    || description.contains("; help ")
                    || Some(description) != property.get("description").and_then(Value::as_str)
                {
                    return Err(format!(
                        "{name}.{key}: lowering enum limit changed the standalone description"
                    ));
                }
            }
        }
    }
    Ok(())
}

fn description(node: &Value) -> Option<String> {
    let text = node
        .get("x-section-rule")
        .or_else(|| node.get("description"))?
        .as_str()?;
    let mut lines = text.lines();
    let first = lines.next().unwrap_or_default();
    let text = if first.contains("; help ") {
        lines
            .next()
            .unwrap_or(first)
            .split("; help ")
            .next()
            .unwrap_or(first)
    } else {
        first
    };
    Some(
        text.split(". ")
            .next()
            .unwrap_or_default()
            .trim_end_matches('.')
            .to_owned()
            + ".",
    )
}
