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
    format!("{verb}_{entity}").replace('-', "_")
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
    let (output_schema, enum_limit) = crate::config::mcp_typed_schema_options(context.settings);
    let excluded = catalog::excluded(context)?;
    let tools = crate::grammar::command_paths()
        .iter()
        .map(|&(verb, entity, id)| {
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
            let mut tool = json!({"name":name(verb, entity),
            "description":description::render(verb, entity, &schema, &excluded),
            "inputSchema":arguments::input_with_limit(&schema, enum_limit),
            "annotations":{"readOnlyHint":read_only,"destructiveHint":false,"openWorldHint":true}});
            if output_schema && let Some(fields) = tool.as_object_mut() {
                fields.insert(
                    "outputSchema".into(),
                    super::output_schema::for_command(verb, entity, id)?,
                );
            }
            let caller_schema = catalog::caller_schema(schema, verb, entity, &excluded);
            let canonical = json!({"name":name(verb, entity),
                "inputSchema":arguments::input_with_limit(&caller_schema, usize::MAX)});
            Ok(Some((tool, canonical)))
        })
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    grouping::catalog(context, read_only, tools, excluded)
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
    let catalog = catalog(context, read_only).ok()?;
    let reference = if params.get("name")?.as_str()? == "biomcp_reference" {
        if !followups::allowed(params, &catalog.tools) {
            return None;
        }
        Some(grouping::unwrap(params)?)
    } else {
        None
    };
    let params = reference.as_ref().unwrap_or(params);
    let called = params.get("name")?.as_str()?;
    if called == "biomcp_help" {
        let mut call = help::parse(params)?;
        call.catalog = Some(catalog);
        return Some(Ok(call));
    }
    if matches!(called, "biomcp" | "search" | "fetch") {
        let mut call = super::research::parse(params)?;
        call.catalog = Some(catalog);
        return Some(Ok(call));
    }
    let &(verb, entity, id) = crate::grammar::command_paths()
        .iter()
        .find(|&&(verb, entity, _)| name(verb, entity) == called)?;
    let schema = schema(id).ok()?;
    let input = arguments::input(&schema);
    let values = params.get("arguments").unwrap_or(&Value::Null);
    if !validation::valid(&input, values) {
        let properties = input.get("properties")?.as_object()?;
        let fields = values.as_object();
        let key =
            fields.and_then(|fields| fields.keys().find(|key| !properties.contains_key(*key)));
        let error = biomcp_types::UsageError::Arguments(biomcp_types::UsageArguments {
            argument: Some(key.map_or_else(|| "arguments".into(), |key| key.clone().into())),
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

pub(super) struct Rejected {
    error: biomcp_types::UsageError,
    args: Vec<std::ffi::OsString>,
    catalog: Option<Catalog>,
}
pub(super) fn rejection(
    id: Value,
    mut rejected: Rejected,
    modern: bool,
    context: crate::command::Context<'_>,
    read_only: bool,
) -> io::Result<Value> {
    if context.globals.hosted {
        let &(verb, entity, id) = crate::grammar::command_paths()
            .iter()
            .find(|(verb, entity, _)| {
                rejected.args.first().is_some_and(|word| word == *verb)
                    && rejected.args.get(1).is_some_and(|word| word == *entity)
            })
            .ok_or_else(|| io::Error::other("typed command path"))?;
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
