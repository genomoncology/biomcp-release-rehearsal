//! Advertised capabilities are projections of command and execution policy.
#[path = "../../../typed_front_map.rs"]
mod front_map;
use biomcp_types::Source;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

pub(in crate::mcp) fn available(
    verb: &str,
    entity: &str,
    schema: &Value,
    excluded: &BTreeSet<Source>,
    hosted: bool,
) -> bool {
    if crate::command::mcp_policy::always_refused(schema, hosted) {
        return false;
    }
    let excluded = capability_excluded(schema, excluded);
    let sources = crate::help::coverage::command_sources(verb, entity, schema);
    !hosted
        || sources.is_empty()
        || sources.iter().any(|word| {
            !word
                .parse::<Source>()
                .is_ok_and(|source| excluded.contains(&source))
        })
}

pub(in crate::mcp) fn excluded(
    context: crate::command::Context<'_>,
) -> io::Result<BTreeSet<Source>> {
    let mut sources = crate::config::excluded_sources(context.globals, context.settings)
        .map_err(|error| io::Error::other(error.0))?;
    sources.extend(crate::command::hosted_use::disabled(
        context.declared_use,
        context.secrets,
    ));
    if context.globals.hosted
        && let Ok(fetch) = context.fetch
    {
        let effective = crate::command::hosted_use::restrict(
            fetch.restrict_sources(Default::default()),
            Some(context),
        );
        sources.extend(
            biomcp_query::sources::components()
                .filter(|component| component.disabled(&effective))
                .filter_map(|component| component.native_source),
        );
    }
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

pub(in crate::mcp) fn section_allowed(
    verb: &str,
    entity: &str,
    section: &str,
    excluded: &BTreeSet<Source>,
) -> bool {
    if biomcp_types::command_section_component(verb, entity, section)
        .is_some_and(|source| excluded.contains(&source))
    {
        return false;
    }
    let candidates = biomcp_query::sources::sources()
        .map(|spec| spec.source)
        .collect::<Vec<_>>();
    biomcp_types::command_section_sources(verb, entity, section, &candidates).is_none_or(
        |sources| {
            sources
                .into_iter()
                .any(|source| !excluded.contains(&source))
        },
    )
}

pub(super) fn hosted_schema(
    mut schema: Value,
    verb: &str,
    entity: &str,
    excluded: &BTreeSet<Source>,
) -> Value {
    let source_exclusions = capability_excluded(&schema, excluded);
    filter(&mut schema, &source_exclusions);
    let allowed = schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|fields| {
            fields
                .keys()
                .filter(|name| argument_allowed(&schema, name, excluded))
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if let Some(properties) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        properties.retain(|name, _| allowed.contains(name));
        properties.remove("save");
        properties.remove("replace");
    }
    filter_sections(&mut schema, verb, entity, excluded);
    schema
}

pub(in crate::mcp) fn capability_excluded(
    schema: &Value,
    excluded: &BTreeSet<Source>,
) -> BTreeSet<Source> {
    let mut result = excluded.clone();
    if let Some(required) = schema
        .pointer("/properties/source/x-required-components")
        .and_then(Value::as_object)
    {
        for (source, component) in required {
            if component
                .as_str()
                .and_then(|word| word.parse::<Source>().ok())
                .is_some_and(|component| excluded.contains(&component))
                && let Ok(source) = source.parse::<Source>()
            {
                result.insert(source);
            }
        }
    }
    result
}

pub(in crate::mcp) fn argument_allowed(
    schema: &Value,
    name: &str,
    excluded: &BTreeSet<Source>,
) -> bool {
    schema
        .get("properties")
        .and_then(|fields| fields.get(name.trim_start_matches('-').replace('-', "_")))
        .and_then(|property| property.get("x-required-component"))
        .and_then(Value::as_str)
        .and_then(|word| word.parse::<Source>().ok())
        .is_none_or(|source| !excluded.contains(&source))
}

pub(in crate::mcp) fn request_allowed(
    request: &biomcp_types::Request,
    excluded: &BTreeSet<Source>,
) -> io::Result<bool> {
    if required_components(request)?
        .iter()
        .any(|component| excluded.contains(component))
    {
        return Ok(false);
    }
    let schema = super::schema(crate::grammar::query_schema(request))?;
    let source_exclusions = capability_excluded(&schema, excluded);
    let query = crate::grammar::rewrite::query_value(request)?;
    let (verb, entity) = request.path();
    for (name, value) in query.as_object().into_iter().flatten() {
        if value.is_null() {
            continue;
        }
        if name == "source"
            && value
                .as_str()
                .and_then(|word| word.parse::<Source>().ok())
                .is_some_and(|source| source_exclusions.contains(&source))
        {
            return Ok(false);
        }
        if !argument_allowed(&schema, name, excluded) {
            return Ok(false);
        }
        if matches!(name.as_str(), "section" | "sections") {
            for section in value
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(std::slice::from_ref(value))
            {
                if section
                    .as_str()
                    .is_some_and(|word| !section_allowed(verb, entity, word, excluded))
                {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

pub(in crate::mcp) fn required_components(
    request: &biomcp_types::Request,
) -> io::Result<BTreeSet<Source>> {
    let schema = super::schema(crate::grammar::query_schema(request))?;
    let query = crate::grammar::rewrite::query_value(request)?;
    let (verb, entity) = request.path();
    let mut required = BTreeSet::new();
    for (name, value) in query
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(_, value)| !value.is_null())
    {
        let property = schema.get("properties").and_then(|fields| fields.get(name));
        if let Some(component) = property
            .and_then(|field| field.get("x-required-component"))
            .and_then(Value::as_str)
            .and_then(|word| word.parse::<Source>().ok())
        {
            required.insert(component);
        }
        for word in value
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(std::slice::from_ref(value))
            .iter()
            .filter_map(Value::as_str)
        {
            if let Some(component) = property
                .and_then(|field| field.get("x-required-components"))
                .and_then(|map| map.get(word))
                .and_then(Value::as_str)
                .and_then(|word| word.parse::<Source>().ok())
            {
                required.insert(component);
            }
            if matches!(name.as_str(), "section" | "sections")
                && let Some(component) = biomcp_types::command_section_component(verb, entity, word)
            {
                required.insert(component);
            }
        }
    }
    Ok(required)
}

fn filter_sections(node: &mut Value, verb: &str, entity: &str, excluded: &BTreeSet<Source>) {
    match node {
        Value::Object(fields) => {
            if let Some(choices) = fields.get_mut("enum").and_then(Value::as_array_mut) {
                choices.retain(|word| {
                    word.as_str()
                        .is_none_or(|word| section_allowed(verb, entity, word, excluded))
                });
            }
            for value in fields.values_mut() {
                filter_sections(value, verb, entity, excluded);
            }
        }
        Value::Array(values) => {
            for value in values {
                filter_sections(value, verb, entity, excluded);
            }
        }
        _ => {}
    }
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
        if !available(verb, entity, &query, &excluded, context.globals.hosted) {
            continue;
        }
        let relevant = relevant(verb, entity, &query, &excluded);
        let mut query = schema(query, &relevant);
        if context.globals.hosted {
            query = hosted_schema(query, verb, entity, &excluded);
        }
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

pub(in crate::mcp) fn summary(text: &str, excluded: &BTreeSet<Source>) -> String {
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
