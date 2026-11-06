//! Summarize typed replies and translate checked CLI Steps to tool calls.
use serde_json::{Value, json};

pub(super) fn render(
    root: &crate::grammar::Root,
    value: &Value,
    args: &[std::ffi::OsString],
    tools: &super::grouping::Catalog,
    projected: &[Value],
) -> String {
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("complete");
    let mut calls = Vec::new();
    for call in projected {
        let mut call = call.clone();
        if let Some(fields) = call.as_object_mut() {
            fields.remove("kind");
        }
        let text = call.to_string();
        if !calls.contains(&text) {
            calls.push(text);
        }
    }
    if calls.is_empty()
        && status == "usage"
        && let Some(call) = correction(root, value, args, tools)
    {
        calls.push(call.to_string());
    }
    calls.first().map_or_else(
        || format!("{status}: JSON reply is in structuredContent."),
        |call| format!("{status}: next {call}"),
    )
}
// The parser names the rejected option; omit it and let the parser check the correction.
fn correction(
    root: &crate::grammar::Root,
    value: &Value,
    args: &[std::ffi::OsString],
    tools: &super::grouping::Catalog,
) -> Option<Value> {
    let option = value.pointer("/error/option")?.as_str()?;
    let mut skip = false;
    let values = args
        .iter()
        .filter_map(|arg| {
            let arg = arg.to_str()?;
            if skip {
                skip = false;
                return None;
            }
            if option.starts_with("--") && arg == option {
                skip = true;
                return None;
            }
            if option.starts_with("--") && arg.starts_with(&format!("{option}=")) {
                return None;
            }
            Some(Value::from(arg))
        })
        .collect::<Vec<_>>();
    translate(root, &values, tools)
}
pub(super) fn translate(
    root: &crate::grammar::Root,
    args: &[Value],
    tools: &super::grouping::Catalog,
) -> Option<Value> {
    let args = args.iter().map(Value::as_str).collect::<Option<Vec<_>>>()?;
    if let Some(call) = super::help::next(root, &args) {
        return super::grouping::call(call, tools);
    }
    let mut argv = vec!["biomcp"];
    argv.extend(args);
    let matches = clap::Command::clone(root).try_get_matches_from(argv).ok()?;
    let (verb, matches) = matches.subcommand()?;
    let (entity, matches) = matches.subcommand()?;
    let (_, _, id) = crate::grammar::command_paths()
        .into_iter()
        .find(|(v, e, _)| *v == verb && *e == entity)?;
    let schema = super::schema(id).ok()?;
    let input = super::arguments::input(&schema);
    let normalized = crate::grammar::request_arguments(verb, entity, matches).ok()?;
    let mut arguments = serde_json::Map::new();
    for key in input.get("properties")?.as_object()?.keys() {
        if let Some(value) = normalized.get(key)
            && (matches.value_source(key) == Some(clap::parser::ValueSource::CommandLine)
                || schema.pointer(&format!("/properties/{key}/x-trailing-section"))
                    == Some(&json!(true)))
        {
            arguments.insert(key.clone(), value.clone());
        } else if input.pointer(&format!("/properties/{key}/type")) == Some(&json!("boolean"))
            && matches.value_source(key) == Some(clap::parser::ValueSource::CommandLine)
        {
            arguments.insert(key.clone(), json!(matches.get_flag(key)));
        }
    }
    let arguments = Value::Object(arguments);
    checked_call(root, verb, entity, &schema, &arguments, tools)
}

fn checked_call(
    root: &crate::grammar::Root,
    verb: &str,
    entity: &str,
    schema: &Value,
    arguments: &Value,
    tools: &super::grouping::Catalog,
) -> Option<Value> {
    if !super::validation::valid(&super::arguments::input(schema), arguments) {
        return None;
    }
    let args = super::arguments::cli(root, verb, entity, schema, arguments)?;
    let mut parsed = crate::grammar::parse(root, &args).ok()?;
    if !tools.permits(&mut parsed) {
        return None;
    }
    let call = json!({"name":super::name(verb,entity),"arguments":arguments});
    if !matches!(parsed.action, Some(biomcp_query::Action::Run { .. })) {
        return None;
    }
    super::grouping::call(call, tools)
}
