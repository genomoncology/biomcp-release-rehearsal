//! Typed tools derive their contracts and CLI spelling from the command registry.
mod arguments;
pub(super) mod catalog;
pub(super) use catalog::instructions;
mod description;
mod followups;
mod grouping;
pub(super) use grouping::Catalog;
mod help;
mod summary;
mod validation;

pub(super) fn prepare(
    outcome: &mut crate::command::Outcome,
    context: crate::command::Context<'_>,
    args: &[std::ffi::OsString],
    read_only: bool,
) -> io::Result<Option<String>> {
    if crate::config::mcp_tool_mode(context.globals, context.settings)
        != crate::config::ToolMode::Typed
    {
        return Ok(None);
    }
    let advertised = catalog(context, read_only)?;
    prepare_with_catalog(outcome, context, args, &advertised)
}

pub(super) fn prepare_with_catalog(
    outcome: &mut crate::command::Outcome,
    context: crate::command::Context<'_>,
    args: &[std::ffi::OsString],
    advertised: &Catalog,
) -> io::Result<Option<String>> {
    if crate::config::mcp_tool_mode(context.globals, context.settings)
        != crate::config::ToolMode::Typed
    {
        return Ok(None);
    }
    let value = outcome.value_mut()?;
    let calls = followups::project(context.root, value, advertised)?;
    Ok(Some(summary::render(
        context.root,
        value,
        args,
        advertised,
        &calls,
        crate::config::mcp_reply_form(context.settings),
    )))
}

pub(super) fn tool_result(
    id: Value,
    completed: &super::research::Completed,
    modern: bool,
    settings: &crate::config::Settings,
) -> Value {
    let form = crate::config::mcp_reply_form(settings);
    let mut reply = super::protocol::tool_result(id, &completed.outcome, modern, form);
    let Some(summary) = completed.summary.as_ref() else {
        return reply;
    };
    let value = reply
        .pointer("/result/structuredContent")
        .cloned()
        .or_else(|| serde_json::from_str(reply.pointer("/result/content/0/text")?.as_str()?).ok());
    if let Some(value) = value {
        let summary = if completed
            .outcome
            .value()
            .is_ok_and(|original| original == &value)
        {
            summary.clone()
        } else {
            format!(
                "{}: {}",
                value
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("complete"),
                super::protocol::reply_note(form)
            )
        };
        let text = if matches!(form, crate::config::ReplyForm::StructuredOnly) {
            summary
        } else {
            format!("{summary}\n{value}")
        };
        if let Some(result) = reply.get_mut("result").and_then(Value::as_object_mut) {
            result.insert("content".into(), json!([{"type":"text","text":text}]));
        }
    }
    reply
}

use serde_json::{Value, json};
use std::io;

fn name(verb: &str, entity: &str) -> String {
    catalog::front_map::command_name(verb, entity)
}
/// Catalog tools that are not entity tools. A test walks the real catalog against this list.
const FIXED_TOOLS: [&str; 4] = ["biomcp", "help", "search", "fetch"];
/// Whether a caller-supplied tool name is one this build can serve, so it is safe to log.
pub(in crate::mcp) fn served(tool: &str) -> bool {
    FIXED_TOOLS.contains(&tool)
        || catalog::front_map::ENTITY_TOOLS
            .iter()
            .any(|(name, _, _)| *name == tool)
}
pub(in crate::mcp) fn schema(id: &str) -> io::Result<Value> {
    serde_json::from_str(
        crate::schema::find(id).ok_or_else(|| io::Error::other("missing query schema"))?,
    )
    .map_err(Into::into)
}
pub(super) fn tools(
    context: crate::command::Context<'_>,
    read_only: bool,
) -> io::Result<Vec<Value>> {
    Ok(catalog(context, read_only)?.tools)
}
fn catalog(context: crate::command::Context<'_>, read_only: bool) -> io::Result<grouping::Catalog> {
    let enum_limit = crate::config::mcp_typed_schema_options(context.settings).1;
    let excluded = catalog::excluded(context)?;
    let commands = crate::grammar::command_paths()
        .into_iter()
        .map(|(verb, entity, id)| {
            let schema = schema(id)?;
            if !catalog::available(verb, entity, &schema, &excluded, context.globals.hosted) {
                return Ok(None);
            }
            let read_only = read_only || !crate::command::mcp_policy::can_save(&schema);
            let relevant = catalog::relevant(verb, entity, &schema, &excluded);
            let mut schema = catalog::schema(schema, &relevant);
            if context.globals.hosted {
                schema = catalog::hosted_schema(schema, verb, entity, &excluded);
            }
            let sources = crate::help::coverage::command_sources(verb, entity, &schema)
                .into_iter()
                .filter(|word| {
                    word.parse::<biomcp_types::Source>()
                        .is_ok_and(|source| !excluded.contains(&source))
                })
                .collect();
            let description = description::render(verb, entity, &schema, &excluded);
            let input = arguments::input_with_limit(&schema, enum_limit);
            let caller_schema = catalog::caller_schema(schema, verb, entity, &excluded);
            let canonical = json!({"name":name(verb, entity),
                "inputSchema":arguments::input_with_limit(&caller_schema, usize::MAX)});
            Ok(Some(grouping::Command {
                path: (verb, entity, id),
                description,
                input,
                read_only,
                sources,
                canonical,
            }))
        })
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    grouping::catalog(context, read_only, commands, excluded)
}
pub(super) fn parse(
    context: crate::command::Context<'_>,
    params: &Value,
    mode: crate::config::ToolMode,
    fresh: bool,
    read_only: bool,
) -> Option<Result<super::research::Call, Rejected>> {
    if mode == crate::config::ToolMode::Single {
        return super::research::parse(params).map(Ok);
    }
    let root = context.root;
    let mut catalog = catalog(context, read_only).ok()?;
    let called = params.get("name")?.as_str()?;
    if called == "help" {
        let mut call = help::parse(params)?;
        call.catalog = Some(catalog);
        return Some(Ok(call));
    }
    if matches!(called, "biomcp" | "search" | "fetch") {
        let mut call = super::research::parse(params)?;
        call.catalog = Some(catalog);
        return Some(Ok(call));
    }
    let (tool, _, placed) = catalog::front_map::ENTITY_TOOLS
        .iter()
        .find(|(tool, _, _)| *tool == called)?;
    catalog.current = Some((*tool).into());
    let fields = match params.get("arguments") {
        Some(Value::Object(fields)) => fields.clone(),
        Some(Value::Null) | None => serde_json::Map::new(),
        Some(_) => return None,
    };
    let action = fields.get("action").and_then(Value::as_str);
    let Some(&(action, command, _)) = placed.iter().find(|(placed, _, _)| Some(*placed) == action)
    else {
        let actions = catalog.actions(tool);
        return Some(Err(rejected("action", &actions, catalog)));
    };
    if let Some(key) = fields.keys().find(|key| *key != "action" && *key != action) {
        return Some(Err(rejected(
            key,
            &["action".into(), action.into()],
            catalog,
        )));
    }
    let path = crate::grammar::command_paths()
        .into_iter()
        .find(|&(verb, entity, _)| name(verb, entity) == command)?;
    let values = fields.get(action).cloned().unwrap_or_else(|| json!({}));
    command_call(root, path, action, &values, fresh, catalog)
}

/// Check one action's object against its command's schema and spell its CLI call.
fn command_call(
    root: &crate::grammar::Root,
    (verb, entity, id): (&'static str, &'static str, &'static str),
    action: &str,
    values: &Value,
    fresh: bool,
    catalog: Catalog,
) -> Option<Result<super::research::Call, Rejected>> {
    let schema = schema(id).ok()?;
    let input = arguments::input(&schema);
    if !validation::valid(&input, values) {
        let properties = input.get("properties")?.as_object()?;
        let fields = values.as_object();
        let key =
            fields.and_then(|fields| fields.keys().find(|key| !properties.contains_key(*key)));
        let error = biomcp_types::UsageError::Arguments(biomcp_types::UsageArguments {
            argument: Some(key.map_or_else(|| action.into(), |key| key.clone().into())),
            valid_values: Some(
                properties
                    .keys()
                    .map(|key| biodata::Text::new(key).ok())
                    .collect::<Option<Vec<_>>>()?
                    .into(),
            ),
            next: vec![biomcp_types::Step {
                kind: biomcp_types::StepKind::Configure,
                args: ["help", verb, entity].into_iter().map(Box::from).collect(),
            }],
            ..Default::default()
        });
        let known = fields
            .map(|fields| {
                Value::Object(
                    fields
                        .iter()
                        .filter(|(key, _)| properties.contains_key(*key))
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect(),
                )
            })
            .unwrap_or_else(|| json!({}));
        let args = arguments::cli(root, verb, entity, &schema, &known)?;
        return Some(Err(Rejected {
            error,
            args,
            command: Some((verb, entity, id)),
            catalog: Some(catalog),
        }));
    }
    let mut args = arguments::cli(root, verb, entity, &schema, values)?;
    if fresh {
        args.insert(2, "--fresh".into());
    }
    Some(Ok(super::research::Call {
        args,
        projection: super::research::Projection::Cli,
        catalog: Some(catalog),
    }))
}

/// Refuse an entity call's own shape: a missing or unknown action, or a field beside it.
fn rejected(argument: &str, valid: &[String], catalog: Catalog) -> Rejected {
    let error = biomcp_types::UsageError::Arguments(biomcp_types::UsageArguments {
        argument: Some(argument.into()),
        valid_values: valid
            .iter()
            .map(|value| biodata::Text::new(value).ok())
            .collect::<Option<Vec<_>>>()
            .map(Into::into),
        next: vec![biomcp_types::Step {
            kind: biomcp_types::StepKind::Configure,
            args: ["help".into()].into(),
        }],
        ..Default::default()
    });
    Rejected {
        error,
        args: vec!["help".into()],
        command: None,
        catalog: Some(catalog),
    }
}

pub(super) struct Rejected {
    error: biomcp_types::UsageError,
    args: Vec<std::ffi::OsString>,
    /// The command whose arguments were refused; none when the entity call's shape was.
    command: Option<(&'static str, &'static str, &'static str)>,
    catalog: Option<Catalog>,
}
pub(super) fn rejection(
    id: Value,
    mut rejected: Rejected,
    modern: bool,
    context: crate::command::Context<'_>,
    read_only: bool,
) -> io::Result<Value> {
    if context.globals.hosted
        && let Some((verb, entity, id)) = rejected.command
    {
        let excluded = catalog::excluded(context)?;
        let schema = catalog::hosted_schema(schema(id)?, verb, entity, &excluded);
        if let biomcp_types::UsageError::Arguments(arguments) = &mut rejected.error
            && let Some(values) = arguments.valid_values.take()
        {
            let values = values
                .into_vec()
                .into_iter()
                .filter(|value| {
                    schema
                        .get("properties")
                        .and_then(|fields| fields.get(value.as_str()))
                        .is_some()
                })
                .collect::<Vec<_>>();
            arguments.valid_values = (!values.is_empty()).then(|| values.into());
        }
    }
    let mut outcome =
        crate::command::usage_envelope(rejected.error, crate::command::Encoding::Mcp)?;
    super::hosted::project(&mut outcome, context, &rejected.args)?;
    let summary = if let Some(catalog) = rejected.catalog.as_ref() {
        prepare_with_catalog(&mut outcome, context, &rejected.args, catalog)?
    } else {
        prepare(&mut outcome, context, &rejected.args, read_only)?
    };
    Ok(tool_result(
        id,
        &super::research::Completed { outcome, summary },
        modern,
        context.settings,
    ))
}
