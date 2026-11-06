//! Typed tools derive their contracts and CLI spelling from the command registry.
mod arguments;
pub(super) mod catalog;
pub(super) use catalog::instructions;
mod description;
mod help;
mod summary;
mod validation;
pub(super) fn tool_result(
    id: Value,
    outcome: &crate::command::Outcome,
    modern: bool,
    globals: &crate::grammar::Globals,
    settings: &crate::config::Settings,
    root: &crate::grammar::Root,
    args: &[std::ffi::OsString],
) -> Value {
    let form = crate::config::mcp_reply_form(settings);
    let reply = super::protocol::tool_result(id, outcome, modern, form);
    summarize(
        reply,
        root,
        crate::config::mcp_tool_mode(globals, settings),
        args,
        form,
    )
}

fn summarize(
    mut reply: Value,
    root: &crate::grammar::Root,
    mode: crate::config::ToolMode,
    args: &[std::ffi::OsString],
    form: crate::config::ReplyForm,
) -> Value {
    if mode != crate::config::ToolMode::Typed {
        return reply;
    }
    let value = reply
        .pointer("/result/structuredContent")
        .cloned()
        .or_else(|| {
            let text = reply.pointer("/result/content/0/text")?.as_str()?;
            serde_json::from_str(text).ok()
        });
    if let Some(value) = value {
        let summary = summary::render(root, &value, args);
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
    let (output_schema, enum_limit) = crate::config::mcp_typed_schema_options(context.settings);
    let excluded = catalog::excluded(context)?;
    let mut tools = crate::grammar::command_paths()
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
            Ok(Some(tool))
        })
        .collect::<io::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    tools.push(help::tool());
    tools.extend(super::research::tools(context.root)?);
    Ok(tools)
}
pub(super) fn parse(
    root: &crate::grammar::Root,
    params: &Value,
    mode: crate::config::ToolMode,
    fresh: bool,
) -> Option<Result<super::research::Call, Rejected>> {
    if mode == crate::config::ToolMode::Single {
        return super::research::parse(params).map(Ok);
    }
    let called = params.get("name")?.as_str()?;
    if called == "biomcp_help" {
        return help::parse(params).map(Ok);
    }
    if matches!(called, "search" | "fetch") {
        return super::research::parse(params).map(Ok);
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
        return Some(Err(Rejected { error, args }));
    }
    let mut args = arguments::cli(root, verb, entity, &schema, values)?;
    if fresh {
        args.insert(2, "--fresh".into());
    }
    Some(Ok(super::research::Call {
        args,
        projection: super::research::Projection::Cli,
    }))
}

pub(super) struct Rejected {
    error: biomcp_types::UsageError,
    args: Vec<std::ffi::OsString>,
}
pub(super) fn rejection(
    id: Value,
    mut rejected: Rejected,
    modern: bool,
    context: crate::command::Context<'_>,
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
    Ok(tool_result(
        id,
        &outcome,
        modern,
        context.globals,
        context.settings,
        context.root,
        &rejected.args,
    ))
}
