//! Advertised capabilities are projections of command and execution policy.
#[path = "../../../typed_front_map.rs"]
mod front_map;
use biomcp_types::Source;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

pub(super) fn excluded(context: crate::command::Context<'_>) -> io::Result<BTreeSet<Source>> {
    let mut sources = crate::config::excluded_sources(context.globals, context.settings)
        .map_err(|error| io::Error::other(error.0))?;
    sources.extend(crate::command::hosted_use::disabled(
        context.declared_use,
        context.secrets,
    ));
    Ok(sources)
}

pub(super) fn relevant(
    verb: &str,
    entity: &str,
    schema: &Value,
    excluded: &BTreeSet<Source>,
) -> BTreeSet<Source> {
    crate::help::coverage::command_sources(verb, entity, schema)
        .into_iter()
        .filter_map(|word| word.parse::<Source>().ok())
        .filter(|source| excluded.contains(source))
        .collect()
}

pub(super) fn schema(mut schema: Value, excluded: &BTreeSet<Source>) -> Value {
    filter(&mut schema, excluded);
    if excluded.is_empty() {
        return schema;
    }
    for key in ["source", "sources", "resolver"] {
        let mut allowed = BTreeSet::new();
        if let Some(node) = schema.pointer(&format!("/properties/{key}")) {
            crate::help::coverage::source_words(node, &schema, &mut allowed);
        }
        if let Some(node) = schema.pointer_mut(&format!("/properties/{key}"))
            && let Some(fields) = node.as_object_mut()
        {
            fields.insert(
                "description".into(),
                json!(
                    format!(
                        "Source: {}",
                        allowed.into_iter().collect::<Vec<_>>().join(",")
                    )
                    .chars()
                    .take(80)
                    .collect::<String>()
                ),
            );
        }
    }
    schema
}

fn filter(node: &mut Value, excluded: &BTreeSet<Source>) {
    match node {
        Value::Object(fields) => {
            if let Some(text) = fields.get("description").and_then(Value::as_str)
                && mentions_excluded(text, excluded)
            {
                fields.insert(
                    "description".into(),
                    json!("See biomcp_help for this option."),
                );
            }
            for key in ["enum", "x-extensible-enum"] {
                if let Some(values) = fields.get_mut(key).and_then(Value::as_array_mut) {
                    values.retain(|value| {
                        value
                            .as_str()
                            .and_then(|word| word.parse::<Source>().ok())
                            .is_none_or(|source| !excluded.contains(&source))
                    });
                }
            }
            for value in fields.values_mut() {
                filter(value, excluded);
            }
        }
        Value::Array(values) => {
            for value in values {
                filter(value, excluded);
            }
        }
        _ => {}
    }
}

pub(in crate::mcp) fn instructions(context: crate::command::Context<'_>) -> io::Result<String> {
    let excluded = excluded(context)?;
    let mut commands = Vec::new();
    let mut schemas = BTreeMap::new();
    for (verb, entity, id) in crate::grammar::command_paths() {
        let query = super::schema(id)?;
        if crate::command::mcp_policy::always_refused(&query, context.globals.hosted) {
            continue;
        }
        let relevant = relevant(verb, entity, &query, &excluded);
        let query = schema(query, &relevant);
        schemas.insert(id.into(), query.to_string());
        commands.push((verb, entity, id));
    }
    front_map::generate(
        &commands,
        &schemas,
        &excluded.iter().map(ToString::to_string).collect(),
        crate::help::coverage::command_sources,
    )
    .map_err(|error| io::Error::other(error.to_string()))
}

pub(super) fn mentions_excluded(text: &str, excluded: &BTreeSet<Source>) -> bool {
    let text = text.to_lowercase();
    names(excluded).any(|name| !matches(&text, &name).is_empty())
}

pub(super) fn summary(text: &str, excluded: &BTreeSet<Source>) -> String {
    let mut text = text.to_owned();
    for name in names(excluded) {
        for index in matches(&text.to_lowercase(), &name).into_iter().rev() {
            text.replace_range(index..index + name.len(), "");
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn names(excluded: &BTreeSet<Source>) -> impl Iterator<Item = String> {
    let mut names = excluded
        .iter()
        .flat_map(|source| {
            [
                source.to_string(),
                biomcp_query::sources::display_name(*source).to_lowercase(),
            ]
            .into_iter()
            .chain(
                biomcp_query::sources::discovery_aliases(*source)
                    .iter()
                    .map(|alias| alias.to_lowercase()),
            )
        })
        .collect::<Vec<_>>();
    names.sort_by_key(|name| std::cmp::Reverse(name.len()));
    names.into_iter()
}

fn matches(text: &str, name: &str) -> Vec<usize> {
    text.match_indices(name)
        .filter(|(index, _)| {
            text.get(..*index)
                .and_then(|prefix| prefix.chars().next_back())
                .is_none_or(|c| !c.is_alphanumeric())
                && text
                    .get(index + name.len()..)
                    .and_then(|tail| tail.chars().next())
                    .is_none_or(|c| !c.is_alphanumeric())
        })
        .map(|(index, _)| index)
        .collect()
}
