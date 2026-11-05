//! Translate recorded command arguments through their existing declarations.
#[path = "../../src/grammar/process_arguments.rs"]
mod process_arguments;
#[path = "../../src/grammar/query_aliases.rs"]
mod query_aliases;
use serde_json::{Value, json};

pub(super) fn query_arguments(args: &[String]) -> (Vec<String>, Vec<String>, Result<(), String>) {
    let parser = process_arguments::command();
    let mut query = Vec::new();
    let mut globals = Vec::new();
    let mut words = args.iter().peekable();
    while let Some(word) = words.next() {
        if word == "--" {
            query.push(word.clone());
            query.extend(words.cloned());
            break;
        }
        let (flag, inline) = word
            .split_once('=')
            .map_or((word.as_str(), false), |(flag, _)| (flag, true));
        let Some(argument) = parser.get_arguments().find(|arg| {
            arg.get_long()
                .is_some_and(|name| flag == format!("--{name}"))
        }) else {
            query.push(word.clone());
            continue;
        };
        let following = words
            .peek()
            .map(|word| std::ffi::OsString::from(word.as_str()));
        if !inline && process_arguments::boolean_value(argument, following.as_ref()) {
            globals.push(format!("{word}={}", words.next().unwrap_or(word)));
        } else {
            globals.push(word.clone());
            if !inline
                && argument.get_action().takes_values()
                && let Some(value) = words.next()
            {
                globals.push(value.clone());
            }
        }
    }
    let validation = process_matches(&globals).map(|_| ());
    (query, globals, validation)
}

fn process_matches(globals: &[String]) -> Result<clap::ArgMatches, String> {
    process_arguments::command()
        .subcommand_required(false)
        .arg_required_else_help(false)
        .try_get_matches_from(std::iter::once("biomcp").chain(globals.iter().map(String::as_str)))
        .map_err(|error| error.to_string())
}

pub(super) fn offline(args: &[String]) -> Result<bool, String> {
    let (_, words, validation) = query_arguments(args);
    validation?;
    Ok(process_matches(&words)?.get_flag("offline"))
}

pub(super) fn request_globals(globals: &[String]) -> Vec<String> {
    globals
        .iter()
        .filter(|word| word.as_str() == "--fresh" || word.starts_with("--fresh="))
        .cloned()
        .collect()
}

pub(super) fn json_globals(globals: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut words = globals.iter();
    while let Some(word) = words.next() {
        if word == "--format" {
            result.push(word.clone());
            if let Some(value) = words.next() {
                result.push(if value == "md" {
                    "json".into()
                } else {
                    value.clone()
                });
            }
        } else if word == "--format=md" {
            result.push("--format=json".into());
        } else {
            result.push(word.clone());
        }
    }
    result
}

fn resolved<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        return root
            .pointer(reference.trim_start_matches('#'))
            .unwrap_or(node);
    }
    if let Some(branch) = node
        .get("anyOf")
        .and_then(Value::as_array)
        .and_then(|v| v.iter().find(|n| n["type"] != "null"))
    {
        return resolved(branch, root);
    }
    node
}
pub(super) fn input(args: &[String], query: &Value, cli: &Value) -> Option<Value> {
    let args = alias_words(args, query, cli)?;
    parse_input(&args, query, cli)
}

fn alias_words(args: &[String], query: &Value, cli: &Value) -> Option<Vec<String>> {
    let aliases = query_aliases::declarations(args.get(1)?, query);
    let mut result = args.get(..2)?.to_vec();
    let mut positional = Vec::new();
    let mut words = args.iter().skip(2).peekable();
    while let Some(word) = words.next() {
        if word == "--" {
            result.push(word.clone());
            result.extend(words.cloned());
            break;
        }
        let (flag, inline) = word
            .split_once('=')
            .map_or((word.as_str(), None), |(flag, value)| (flag, Some(value)));
        let Some(alias) = aliases.iter().find(|alias| {
            flag == format!("-{}", alias.short)
                || alias.long.is_some_and(|long| flag == format!("--{long}"))
        }) else {
            result.push(word.clone());
            continue;
        };
        let value = option_word(&mut words, inline, cli)?;
        if alias.argument != alias.field {
            positional.push(value);
        } else {
            result.push(format!("--{}={value}", alias.field.replace('_', "-")));
        }
    }
    if !positional.is_empty() {
        if !result.iter().any(|word| word == "--") {
            result.push("--".into());
        }
        result.extend(positional);
    }
    Some(result)
}

fn option_word<'a>(
    words: &mut std::iter::Peekable<impl Iterator<Item = &'a String>>,
    inline: Option<&str>,
    cli: &Value,
) -> Option<String> {
    if let Some(value) = inline {
        return Some(value.to_owned());
    }
    let value = words.next()?.clone();
    if value == "--"
        || value.starts_with('-') && value != "-" && value.parse::<f64>().is_err()
        || value.starts_with('-')
            && cli.pointer("/error/option").and_then(Value::as_str) == Some(value.as_str())
    {
        return None;
    }
    Some(value)
}

fn parse_input(args: &[String], query: &Value, cli: &Value) -> Option<Value> {
    let properties = query["properties"].as_object()?;
    let mut result = serde_json::Map::new();
    let mut positional = Vec::new();
    let mut words = args.iter().skip(2).peekable();
    while let Some(word) = words.next() {
        if word == "--" {
            positional.extend(words.cloned());
            break;
        }
        let Some(flag) = word.strip_prefix("--") else {
            if word.starts_with('-') {
                return None;
            }
            positional.push(word.clone());
            continue;
        };
        let (flag, inline) = flag
            .split_once('=')
            .map_or((flag, None), |(a, b)| (a, Some(b)));
        let key = flag.replace('-', "_");
        if properties.values().any(|property| {
            property
                .get("x-cli-flags")
                .and_then(|flags| flags.get(flag))
                .is_some()
        }) {
            if inline.is_some() {
                return None;
            }
            result.insert(flag.into(), json!(true));
            continue;
        }
        let property = properties.get(&key)?;
        if property.get("x-positional").is_some() {
            return None;
        }
        let kind = type_name(resolved(property, query));
        if kind == Some("array") {
            let word = option_word(&mut words, inline, cli)?;
            let values = array_words(&mut words, word, property);
            result
                .entry(key)
                .or_insert(json!([]))
                .as_array_mut()?
                .extend(values);
            continue;
        }
        let value = scalar_word(&mut words, inline, kind, cli)?;
        result.insert(key, value);
    }
    positional_fields(properties, query, result, &positional)
}

fn scalar_word<'a>(
    words: &mut std::iter::Peekable<impl Iterator<Item = &'a String>>,
    inline: Option<&str>,
    kind: Option<&str>,
    cli: &Value,
) -> Option<Value> {
    if kind == Some("boolean") {
        let word = inline
            .map(str::to_owned)
            .or_else(|| {
                words
                    .peek()
                    .filter(|word| matches!(word.as_str(), "true" | "false"))?;
                words.next().cloned()
            })
            .unwrap_or_else(|| "true".into());
        return Some(
            word.parse::<bool>()
                .map_or_else(|_| json!(word), |value| json!(value)),
        );
    }
    let word = option_word(words, inline, cli)?;
    Some(match kind {
        Some("integer") => word
            .parse::<u64>()
            .map_or_else(|_| json!(word), |value| json!(value)),
        Some("number") => word
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map_or_else(|| json!(word), |value| json!(value)),
        _ => json!(word),
    })
}

fn array_words<'a>(
    words: &mut std::iter::Peekable<impl Iterator<Item = &'a String>>,
    word: String,
    property: &Value,
) -> Vec<Value> {
    let mut values = if property["x-comma-separated"] == true {
        word.split(',').map(str::to_owned).collect::<Vec<_>>()
    } else {
        vec![word]
    };
    if property["x-comma-separated"] != true && property["x-trailing-section"] != true {
        while words.peek().is_some_and(|value| !value.starts_with('-')) {
            if let Some(value) = words.next() {
                values.push(value.clone());
            }
        }
    }
    values.into_iter().map(Value::String).collect()
}

fn positional_fields(
    properties: &serde_json::Map<String, Value>,
    query: &Value,
    mut result: serde_json::Map<String, Value>,
    positional: &[String],
) -> Option<Value> {
    let mut fields = properties
        .iter()
        .filter_map(|(key, p)| p["x-positional"].as_u64().map(|i| (i, key, p)))
        .collect::<Vec<_>>();
    fields.sort_by_key(|(index, _, _)| *index);
    let mut position = 0;
    for (_, key, property) in fields {
        let kind = type_name(resolved(property, query));
        if kind == Some("array") {
            result.insert(key.clone(), json!(positional.get(position..)?));
            position = positional.len();
        } else if let Some(value) = positional.get(position) {
            result.insert(key.clone(), json!(value));
            position += 1;
        }
    }
    (position == positional.len()).then_some(Value::Object(result))
}

fn type_name(node: &Value) -> Option<&str> {
    node["type"].as_str().or_else(|| {
        node["type"]
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .find(|kind| *kind != "null")
    })
}
