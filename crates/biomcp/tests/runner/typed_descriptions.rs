//! Installed discovery uses command-owned summaries, primary operands and section publishers.
use crate::runner::{
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(super) fn check(env: &Environment) -> Result<(), String> {
    let replies = exchange_with_globals(
        env,
        &[request(json!(1), "tools/list", json!({}))],
        &["--offline".into()],
    )?;
    let tools = replies
        .first()
        .and_then(|r| r.pointer("/result/tools"))
        .and_then(Value::as_array)
        .ok_or("tools")?;
    faers_drug(tools)?;
    for (verb, entity, id) in biomcp_types::command_paths() {
        let name = format!("{verb}_{entity}").replace('-', "_");
        let tool = tools
            .iter()
            .find(|tool| tool.get("name") == Some(&json!(name)))
            .ok_or("registry tool")?;
        let description = tool
            .get("description")
            .and_then(Value::as_str)
            .ok_or("description")?;
        let summary = biomcp_types::COMMANDS
            .iter()
            .find(|row| row.verb == verb && row.entity == entity)
            .ok_or("registry summary")?
            .summary;
        if summary != summary.trim()
            || summary.is_empty()
            || summary.lines().count() != 1
            || !description.starts_with(summary)
            || description.chars().count() > 300
        {
            return Err(format!(
                "{name}: command summary or budget differs: {description}"
            ));
        }
        let schema = cli(env, &["schema".into(), id.into()])?;
        primary(&schema, description, &name)?;
        publishers(verb, entity, &schema, description)?;
    }
    for (name, words) in [
        ("get_drug", ["drug", "medicine"]),
        ("search_adverse_event", ["adverse", "events"]),
        ("search_variant", ["genetic", "variants"]),
    ] {
        let description = tools
            .iter()
            .find(|t| t.get("name") == Some(&json!(name)))
            .and_then(|t| t.get("description"))
            .and_then(Value::as_str)
            .ok_or("spot description")?;
        if !words.into_iter().all(|word| description.contains(word)) {
            return Err(format!("{name}: missing search words"));
        }
    }
    let help = cli(env, &["help".into(), "get".into(), "drug".into()])?;
    let description = help
        .pointer("/result/commands/0/description")
        .and_then(Value::as_str)
        .ok_or("drug help")?;
    let summary = biomcp_types::COMMANDS
        .iter()
        .find(|row| row.verb == "get" && row.entity == "drug")
        .ok_or("drug registry")?
        .summary
        .trim();
    if !description.starts_with(summary) {
        return Err("CLI help lost the command summary".into());
    }
    Ok(())
}
fn faers_drug(tools: &[Value]) -> Result<(), String> {
    let description = tools
        .iter()
        .find(|tool| tool.get("name") == Some(&json!("search_adverse_event")))
        .and_then(|tool| tool.pointer("/inputSchema/properties/drug/description"))
        .and_then(Value::as_str)
        .ok_or("typed FAERS drug description")?;
    if description != "FAERS drug names; OR." {
        return Err(format!(
            "typed FAERS drug lost its owning description: {description}"
        ));
    }
    Ok(())
}
fn cli(env: &Environment, args: &[String]) -> Result<Value, String> {
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("discovery command failed".into());
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
fn primary(schema: &Value, description: &str, name: &str) -> Result<(), String> {
    let primary = schema
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(name, node)| {
            node.get("x-positional")
                .and_then(Value::as_u64)
                .map(|p| (p, name))
        })
        .min_by_key(|(position, _)| *position);
    if let Some((_, input)) = primary
        && !description.contains(&format!("{input}:"))
    {
        return Err(format!(
            "{name}: primary input {input} is missing: {description}"
        ));
    }
    Ok(())
}
fn publishers(verb: &str, entity: &str, schema: &Value, description: &str) -> Result<(), String> {
    let mut sections = BTreeSet::new();
    for key in ["section", "sections"] {
        if let Some(node) = schema.get("properties").and_then(|p| p.get(key)) {
            words(node, schema, &mut sections);
        }
    }
    for section in sections.iter().filter(|section| section.as_str() != "all") {
        let sources =
            biomcp_types::command_section_sources(verb, entity, section, biomcp_types::Source::ALL)
                .unwrap_or_default();
        if !description.contains(section)
            || sources
                .iter()
                .any(|source| !description.contains(&source.to_string()))
        {
            return Err(format!(
                "{verb} {entity}: {section} publisher missing: {description}"
            ));
        }
    }
    Ok(())
}
fn words(node: &Value, root: &Value, out: &mut BTreeSet<String>) {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str)
        && let Some(target) = root.pointer(reference.trim_start_matches('#'))
    {
        words(target, root, out);
    }
    out.extend(
        node.get("enum")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned),
    );
    if let Some(items) = node.get("items") {
        words(items, root, out);
    }
    for key in ["anyOf", "oneOf"] {
        for node in node
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            words(node, root, out);
        }
    }
}
