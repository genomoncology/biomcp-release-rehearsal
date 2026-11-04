//! Reverse the same embedded query schema used by the parser.
#![deny(clippy::wildcard_enum_match_arm)]

use biomcp_query::Action;
use biomcp_types::{Request, Step, StepKind};
use serde_json::Value;

pub(crate) fn step(kind: StepKind, action: &Action) -> Result<Step, serde_json::Error> {
    Ok(Step {
        kind,
        args: action_args(action)?
            .into_iter()
            .map(String::into_boxed_str)
            .collect(),
    })
}
pub(crate) fn action_args(action: &Action) -> Result<Vec<String>, serde_json::Error> {
    match action {
        Action::NetworkHelp(setting) => {
            Ok(vec!["help".into(), "network".into(), setting.to_string()])
        }
        Action::PubmedSearch(q) => {
            let mut args = request_args(&Request::SearchArticle(q.clone()))?;
            if !args.iter().any(|arg| arg == "--source") {
                let position = args
                    .iter()
                    .position(|arg| arg == "--")
                    .unwrap_or(args.len());
                args.splice(position..position, ["--source".into(), "pubmed".into()]);
            }
            Ok(args)
        }
        Action::AuthorPapers(q) => request_args(&Request::AuthorPapers(q.clone())),
        Action::QueryHelp(request) => Ok(std::iter::once("help".into())
            .chain(command_words(request))
            .collect()),
        Action::StoreClean => Ok(vec!["store".into(), "clean".into()]),
        Action::Health => Ok(vec!["health".into()]),
        Action::KeyHelp(source) => Ok(vec![
            "help".into(),
            "key".into(),
            biomcp_query::sources::key_name(*source).into(),
        ]),
        Action::Sync { bundle } => Ok(vec!["sync".into(), bundle.name().into()]),
        Action::Run { request, fresh } => {
            let mut args = request_args(request)?;
            if *fresh {
                let position = args
                    .iter()
                    .position(|arg| arg == "--")
                    .unwrap_or(args.len());
                args.insert(position, "--fresh".into());
            }
            Ok(args)
        }
    }
}
fn request_args(request: &Request) -> Result<Vec<String>, serde_json::Error> {
    let schema: Value = serde_json::from_str(
        crate::schema::find(crate::grammar::query_schema(request)).unwrap_or("{}"),
    )?;
    let query = crate::grammar::rewrite::query_value(request)?;
    let mut args = command_words(request);
    let positional = fields(&schema, &query, &mut args);
    if positional.iter().any(|arg| arg.starts_with('-')) {
        args.push("--".into());
        args.extend(positional);
    } else {
        args.splice(2..2, positional);
    }
    Ok(args)
}

fn trailing_input(schema: &Value, query: &Value, property: &Value) -> bool {
    let item = property.get("items").unwrap_or(&Value::Null);
    let item = item
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|s| schema.pointer(s.trim_start_matches('#')))
        .unwrap_or(item);
    query
        .get("input")
        .and_then(Value::as_array)
        .is_some_and(|inputs| {
            inputs.iter().any(|input| {
                item.get("enum")
                    .and_then(Value::as_array)
                    .is_some_and(|words| words.contains(input))
            })
        })
}

pub(crate) fn command_words(request: &Request) -> Vec<String> {
    let (verb, entity) = request.path();
    vec![verb.into(), entity.into()]
}
pub(crate) fn command_entity(request: &Request) -> &'static str {
    request.path().1
}

fn fields(schema: &Value, query: &Value, args: &mut Vec<String>) -> Vec<String> {
    let mut positional = Vec::new();
    let mut trailing = Vec::new();
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        let mut fields: Vec<_> = properties.iter().collect();
        fields.sort_by_key(|(_, property)| {
            property
                .get("x-positional")
                .and_then(Value::as_u64)
                .unwrap_or(u64::MAX)
        });
        for (name, property) in fields {
            let Some(value) = query
                .get(name)
                .filter(|value| !value.is_null() && property.get("default") != Some(*value))
            else {
                continue;
            };
            if value.is_boolean() {
                if value == &Value::Bool(true) {
                    args.push(format!("--{}", name.replace('_', "-")));
                }
                continue;
            }
            let is_trailing = property.get("x-trailing-section") == Some(&Value::Bool(true));
            let explicit = is_trailing && trailing_input(schema, query, property);
            if property.get("x-positional").is_none() && !is_trailing {
                args.push(format!("--{}", name.replace('_', "-")));
            }
            if property.get("x-comma-separated") == Some(&Value::Bool(true)) {
                args.push(
                    value
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(","),
                );
                continue;
            }
            for value in value
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(std::slice::from_ref(value))
            {
                let target = if is_trailing && !explicit {
                    &mut trailing
                } else if property.get("x-positional").is_some() {
                    &mut positional
                } else {
                    &mut *args
                };
                if explicit {
                    target.push(format!("--{}", name.replace('_', "-")));
                }
                target.push(
                    value
                        .as_str()
                        .map_or_else(|| value.to_string(), str::to_owned),
                );
            }
        }
    }
    positional.extend(trailing);
    positional
}
