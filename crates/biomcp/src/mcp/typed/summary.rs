//! Keep the CLI reply unchanged while spelling executable next calls in the text summary.
use serde_json::{Value, json};

pub(super) fn render(
    root: &crate::grammar::Root,
    value: &Value,
    args: &[std::ffi::OsString],
) -> String {
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("complete");
    let mut calls = Vec::new();
    collect(root, value, &mut calls);
    if calls.is_empty()
        && status == "usage"
        && let Some(call) = correction(root, value, args)
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
    translate(root, &values)
}
fn collect(root: &crate::grammar::Root, value: &Value, calls: &mut Vec<String>) {
    if let Some(args) = value.get("args").and_then(Value::as_array)
        && let Some(call) = translate(root, args)
    {
        let text = call.to_string();
        if !calls.contains(&text) && calls.len() < 3 {
            calls.push(text);
        }
    }
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                // Only protocol-owned follow-ups are interpreted; source text remains data.
                if matches!(key.as_str(), "next" | "error" | "problems") {
                    collect(root, value, calls);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                collect(root, value, calls);
            }
        }
        _ => {}
    }
}
fn translate(root: &crate::grammar::Root, args: &[Value]) -> Option<Value> {
    let args = args.iter().map(Value::as_str).collect::<Option<Vec<_>>>()?;
    if let Some(call) = super::help::next(root, &args) {
        return Some(call);
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
    let command = root.find_subcommand(verb)?.find_subcommand(entity)?;
    let mut arguments = serde_json::Map::new();
    for (key, property) in input.get("properties")?.as_object()? {
        if matches.value_source(key) != Some(clap::parser::ValueSource::CommandLine) {
            continue;
        }
        let arg = command
            .get_arguments()
            .find(|arg| arg.get_id().as_str() == key)?;
        let value = if matches!(arg.get_action(), clap::ArgAction::SetTrue) {
            Value::Bool(matches.get_flag(key))
        } else {
            let values = matches
                .get_raw(key)?
                .map(|v| v.to_str())
                .collect::<Option<Vec<_>>>()?;
            let kind = non_null_type(property, &input);
            if kind == Some("array") {
                json!(values)
            } else {
                let word = values.first()?;
                if kind == Some("integer") {
                    json!(word.parse::<u64>().ok()?)
                } else {
                    json!(word)
                }
            }
        };
        arguments.insert(key.clone(), value);
    }
    let arguments = Value::Object(arguments);
    checked_call(root, verb, entity, &schema, &arguments)
}

fn checked_call(
    root: &crate::grammar::Root,
    verb: &str,
    entity: &str,
    schema: &Value,
    arguments: &Value,
) -> Option<Value> {
    if !super::validation::valid(&super::arguments::input(schema), arguments) {
        return None;
    }
    let args = super::arguments::cli(root, verb, entity, schema, arguments)?;
    let parsed = crate::grammar::parse(root, &args).ok()?;
    matches!(parsed.action, Some(biomcp_query::Action::Run { .. }))
        .then(|| json!({"name":super::name(verb,entity),"arguments":arguments}))
}
fn non_null_type<'a>(node: &'a Value, root: &'a Value) -> Option<&'a str> {
    let node = super::arguments::resolve(node, root);
    node["type"]
        .as_str()
        .or_else(|| {
            node["type"]
                .as_array()?
                .iter()
                .filter_map(Value::as_str)
                .find(|kind| *kind != "null")
        })
        .or_else(|| {
            node["anyOf"]
                .as_array()?
                .iter()
                .filter_map(|node| non_null_type(node, root))
                .find(|kind| *kind != "null")
        })
}
