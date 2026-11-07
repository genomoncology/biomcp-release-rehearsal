//! Source and section provenance comes from the same declarations as the front map.
use crate::help::coverage;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn render(
    verb: &str,
    entity: &str,
    schema: &Value,
    excluded: &BTreeSet<biomcp_types::Source>,
) -> String {
    let sources = coverage::command_sources(verb, entity, schema)
        .into_iter()
        .filter(|word| {
            word.parse::<biomcp_types::Source>()
                .is_ok_and(|source| !excluded.contains(&source))
        })
        .collect::<BTreeSet<_>>();
    let provenance = provenance(verb, entity, schema, &sources, excluded);
    let summary = biomcp_types::COMMANDS
        .iter()
        .find(|row| row.verb == verb && row.entity == entity)
        .map_or("", |row| row.summary.trim());
    let summary = super::catalog::summary(summary, excluded);
    let input = input_description(schema);
    [summary.as_str(), input.as_str(), provenance.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        + "."
}

fn provenance(
    verb: &str,
    entity: &str,
    schema: &Value,
    sources: &BTreeSet<String>,
    excluded: &BTreeSet<biomcp_types::Source>,
) -> String {
    let candidates = sources
        .iter()
        .filter_map(|source| source.parse().ok())
        .collect::<Vec<_>>();
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for key in ["section", "sections"] {
        let node = super::arguments::resolve(
            schema
                .get("properties")
                .and_then(|fields| fields.get(key))
                .unwrap_or(&Value::Null),
            schema,
        );
        let node = node
            .get("anyOf")
            .and_then(Value::as_array)
            .and_then(|branches| branches.iter().find(|node| node["type"] != "null"))
            .map(|node| super::arguments::resolve(node, schema))
            .unwrap_or(node);
        let node = super::arguments::resolve(node.get("items").unwrap_or(node), schema);
        for section in node
            .get("enum")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|word| *word != "all")
        {
            let publishers =
                biomcp_types::command_section_sources(verb, entity, section, &candidates)
                    .map(|sources| {
                        sources
                            .into_iter()
                            .filter(|source| !excluded.contains(source))
                            .map(|s| s.to_string())
                            .collect::<BTreeSet<_>>()
                    })
                    .unwrap_or_else(|| sources.clone());
            if publishers.is_empty() {
                continue;
            }
            groups
                .entry(publishers.into_iter().collect::<Vec<_>>().join(","))
                .or_default()
                .push(section.into());
        }
    }
    if groups.is_empty() {
        sources.iter().cloned().collect::<Vec<_>>().join(",")
    } else {
        let mut same = Vec::new();
        let mut other = Vec::new();
        for (sources, sections) in groups {
            if sections.len() == 1 && sections.first() == Some(&sources) {
                same.push(sources);
            } else {
                other.push(format!("{}:{sources}", sections.join(",")));
            }
        }
        same.extend(other);
        format!("Sections:{}", same.join(";"))
    }
}

fn input_description(schema: &Value) -> String {
    let Some((_, name, node)) = schema
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(name, node)| {
            node.get("x-positional")
                .and_then(Value::as_u64)
                .map(|position| (position, name, node))
        })
        .min_by_key(|(position, _, _)| *position)
    else {
        return String::new();
    };
    let description = node
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .lines()
        .next()
        .unwrap_or_default()
        .split(". ")
        .next()
        .unwrap_or_default()
        .split(';')
        .next()
        .unwrap_or_default();
    let description = description
        .replace("identifiers", "IDs")
        .replace("reference-qualified ", "")
        .replace("VCF-style genomic coordinates", "VCF")
        .replace(", in input order", "")
        .replace(", in caller order", "");
    format!("{name}:{description}")
}

/// One line naming the entity, its sources and every action, then how to call it.
pub(super) fn tool(
    label: &str,
    sources: &BTreeSet<String>,
    actions: &[&str],
    facets: &[&str],
    excluded: &BTreeSet<biomcp_types::Source>,
) -> String {
    let label = super::catalog::summary(label, excluded);
    let lead = if sources.is_empty() {
        label
    } else {
        format!(
            "{label} ({})",
            sources.iter().cloned().collect::<Vec<_>>().join(", ")
        )
    };
    let example = if actions.contains(&"get") {
        "get"
    } else {
        actions.first().copied().unwrap_or("get")
    };
    let pages = if actions.contains(&"get") && !facets.is_empty() {
        format!(
            " get returns the first page of a section; {} return all of it.",
            facets.join(", ")
        )
    } else {
        String::new()
    };
    format!(
        "{lead}. Actions: {}.{pages} Call {{\"action\":\"{example}\",\"{example}\":{{...}}}}.",
        actions.join(", ")
    )
}

/// An action's object keeps its command's description; citations says what it lists.
pub(super) fn action(action: &str, description: &str) -> String {
    if action == "citations" {
        format!("{description} Lists citing papers, not counts.")
    } else {
        description.to_owned()
    }
}
