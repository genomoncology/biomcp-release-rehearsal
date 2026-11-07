//! Project hosted discovery and recovery through the same execution authority.
use crate::command::Context;
use biomcp_types::{Help, ResponseBody, Source};
use serde_json::{Value, json};
use std::{ffi::OsString, io};

/// Ordinary HTTP also refuses synchronization and fresh reads; share execution's authority.
pub(super) fn transport_next(
    out: &mut crate::command::Outcome,
    cx: Context<'_>,
    http: bool,
) -> io::Result<()> {
    if http && !cx.globals.hosted {
        next(out.value_mut()?, cx, false);
    }
    Ok(())
}

pub(super) fn project(
    out: &mut crate::command::Outcome,
    cx: Context<'_>,
    args: &[OsString],
) -> io::Result<()> {
    if !cx.globals.hosted {
        return Ok(());
    }
    let value = out.value_mut()?;
    help(value, cx)?;
    recovery(value, cx)?;
    next(value, cx, true);
    let required = match crate::grammar::parse(cx.root, args)
        .ok()
        .and_then(|parsed| parsed.action)
    {
        Some(biomcp_query::Action::Run { request, .. }) => {
            super::typed::catalog::required_components(&request)?
        }
        _ => Default::default(),
    };
    notices(value, cx, &required)
}

fn help(value: &mut Value, cx: Context<'_>) -> io::Result<()> {
    if value.get("schema") == Some(&json!(Help::SCHEMA)) {
        let mut help: Help = serde_json::from_value(value["result"].clone())?;
        help.commands = commands(help.commands, cx)?;
        if help
            .commands
            .iter()
            .any(|command| command.command == ["help", "relations"])
        {
            let available = commands(crate::help::commands(cx.root)?, cx)?;
            let description = crate::help::relations(&available);
            for relation in help
                .commands
                .iter_mut()
                .filter(|command| command.command == ["help", "relations"])
            {
                relation.description = description.clone();
            }
        }
        value["result"] = serde_json::to_value(help)?;
    }
    Ok(())
}

fn commands(
    mut commands: Vec<biomcp_types::HelpCommand>,
    cx: Context<'_>,
) -> io::Result<Vec<biomcp_types::HelpCommand>> {
    let excluded = super::typed::catalog::excluded(cx)?;
    let mut offered_commands = Vec::new();
    for command in commands {
        if offered(&command.command, cx, &excluded)? {
            offered_commands.push(command);
        }
    }
    commands = offered_commands;
    for command in &mut commands {
        let schema = crate::grammar::command_paths()
            .iter()
            .find(|(verb, entity, _)| command.command == [*verb, *entity])
            .map(|(_, _, id)| super::typed::schema(id))
            .transpose()?;
        let command_exclusions = schema
            .as_ref()
            .map(|schema| super::typed::catalog::capability_excluded(schema, &excluded))
            .unwrap_or_else(|| excluded.clone());
        if command.command.is_empty() {
            command.description = if crate::config::mcp_tool_mode(cx.globals, cx.settings)
                == crate::config::ToolMode::Typed
            {
                super::typed::catalog::instructions(cx)?
            } else {
                instructions(cx)?
            };
        }
        if command.command == ["help", "sources"] || command.command == ["help", "config"] {
            command.description = sources(cx)?;
        } else {
            command.description =
                super::typed::catalog::summary(&command.description, &command_exclusions);
        }
        command.arguments.retain(|argument| {
            schema.as_ref().is_none_or(|schema| {
                super::typed::catalog::argument_allowed(schema, &argument.name, &excluded)
            }) && offered_argument(&argument.name)
        });
        if let [verb, entity] = command.command.as_slice() {
            command.sections.retain(|section| {
                super::typed::catalog::section_allowed(verb, entity, &section.word, &excluded)
            });
        }
        for argument in &mut command.arguments {
            argument.description = super::typed::catalog::summary(&argument.description, &excluded);
            answered_by(argument, schema.as_ref(), &excluded);
            if matches!(
                argument.name.trim_start_matches('-'),
                "section" | "sections"
            ) && let [verb, entity] = command.command.as_slice()
            {
                argument.choices.retain(|word| {
                    super::typed::catalog::section_allowed(verb, entity, word, &excluded)
                });
                if argument.default.as_ref().is_some_and(|word| {
                    !super::typed::catalog::section_allowed(verb, entity, word, &excluded)
                }) {
                    argument.default = None;
                }
            }
            argument.choices.retain(|word| {
                !word
                    .parse::<Source>()
                    .is_ok_and(|source| command_exclusions.contains(&source))
            });
            if argument.default.as_ref().is_some_and(|word| {
                word.parse::<Source>()
                    .is_ok_and(|source| command_exclusions.contains(&source))
            }) {
                argument.default = None;
            }
        }
        command.examples.retain(|args| next_allowed(args, cx));
    }
    Ok(commands)
}

// Name the served sources that answer an argument, so a caller sees what it needs.
fn answered_by(
    argument: &mut biomcp_types::HelpArgument,
    schema: Option<&Value>,
    excluded: &std::collections::BTreeSet<Source>,
) {
    let served = schema
        .iter()
        .flat_map(|schema| super::typed::catalog::argument_sources(schema, &argument.name))
        .filter(|source| !excluded.contains(source))
        .map(|source| source.to_string())
        .collect::<Vec<_>>();
    if !served.is_empty() {
        argument.description = format!(
            "{} Answered by: {}.",
            argument.description,
            served.join(", ")
        );
    }
}

fn offered_argument(name: &str) -> bool {
    !matches!(
        name,
        "--fresh"
            | "--config"
            | "--source_pacing"
            | "--save"
            | "--replace"
            | "--replay"
            | "--record"
            | "--case-cache"
            | "--observe"
    )
}

fn recovery(value: &mut Value, cx: Context<'_>) -> io::Result<()> {
    let path = value
        .get("next")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|step| step.get("args").and_then(Value::as_array))
        .find(|words| words.first() == Some(&json!("help")))
        .and_then(|words| {
            crate::grammar::command_paths()
                .iter()
                .find(|(verb, entity, _)| {
                    words.get(1) == Some(&json!(verb)) && words.get(2) == Some(&json!(entity))
                })
                .copied()
        });
    let schema = path
        .map(|(_, _, id)| super::typed::schema(id))
        .transpose()?;
    let excluded = super::typed::catalog::excluded(cx)?;
    let source_exclusions = schema
        .as_ref()
        .map(|schema| super::typed::catalog::capability_excluded(schema, &excluded))
        .unwrap_or_else(|| excluded.clone());
    let option = value
        .pointer("/error/option")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let field_allowed = schema
        .as_ref()
        .is_none_or(|schema| super::typed::catalog::argument_allowed(schema, &option, &excluded));
    if let Some(values) = value
        .pointer_mut("/error/valid_values")
        .and_then(Value::as_array_mut)
    {
        values.retain(|value| {
            field_allowed
                && value.as_str().is_none_or(|word| {
                    if word.starts_with("--") {
                        return offered_argument(word)
                            && schema.as_ref().is_none_or(|schema| {
                                super::typed::catalog::argument_allowed(schema, word, &excluded)
                            });
                    }
                    match option.trim_start_matches('-') {
                        "source" | "sources" | "resolver" => !word
                            .parse::<Source>()
                            .is_ok_and(|source| source_exclusions.contains(&source)),
                        "section" | "sections" => path.is_none_or(|(verb, entity, _)| {
                            super::typed::catalog::section_allowed(verb, entity, word, &excluded)
                        }),
                        _ => true,
                    }
                })
        });
        if values.is_empty()
            && let Some(error) = value.get_mut("error").and_then(Value::as_object_mut)
        {
            error.remove("valid_values");
        }
    }
    Ok(())
}

fn next(value: &mut Value, cx: Context<'_>, drop_empty: bool) {
    // Envelope recovery commands are Steps; source-body continuations are queries.
    if let Some(steps) = value.get_mut("next").and_then(Value::as_array_mut) {
        let offered = !steps.is_empty();
        steps.retain_mut(|step| next_step(step, cx));
        // Ordinary HTTP leaves an already-empty list as stdio shows it; only a list this filter emptied goes.
        if (drop_empty || offered)
            && steps.is_empty()
            && let Some(fields) = value.as_object_mut()
        {
            fields.remove("next");
        }
    }
}

fn next_step(step: &mut Value, cx: Context<'_>) -> bool {
    let Some(args) = step.get_mut("args").and_then(Value::as_array_mut) else {
        return false;
    };
    // Inherited server globals precede any positional end-of-options separator.
    let mut index = 0;
    while let Some(word) = args.get(index) {
        if word == "--" {
            break;
        }
        if matches!(
            word.as_str(),
            Some("--config" | "--replay" | "--record" | "--case-cache" | "--observe")
        ) {
            let end = (index + 2).min(args.len());
            args.drain(index..end);
        } else {
            index += 1;
        }
    }
    let Some(words) = args
        .iter()
        .map(|v| v.as_str().map(String::from))
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    next_allowed(&words, cx)
}

fn notices(
    value: &mut Value,
    cx: Context<'_>,
    required: &std::collections::BTreeSet<Source>,
) -> io::Result<()> {
    if let Some(rows) = value.get_mut("sources").and_then(Value::as_array_mut) {
        for row in rows {
            // A row with its own notice, such as one left-out study, has a source that stays on.
            let explained = row
                .get("notices")
                .and_then(Value::as_array)
                .is_some_and(|rows| !rows.is_empty());
            if row.get("reason") != Some(&json!("disabled")) || explained {
                continue;
            }
            let source: Source = serde_json::from_value(row["source"].clone())?;
            let notice = serde_json::to_value(biomcp_types::SourceNotice::RequestGuidance {
                data_date: None,
                message: reason(source, cx, required)?.into(),
                retry_after_seconds: None,
            })?;
            if let Some(fields) = row.as_object_mut() {
                fields
                    .entry("notices")
                    .or_insert_with(|| json!([]))
                    .as_array_mut()
                    .ok_or_else(|| io::Error::other("source notices"))?
                    .push(notice);
            }
        }
    }
    Ok(())
}

pub(super) fn instructions(cx: Context<'_>) -> io::Result<String> {
    let excluded = super::typed::catalog::excluded(cx)?;
    let mut rows = vec!["Use the biomcp tool with command words in args; use help for arguments and help sources for this server's source policy.".to_owned()];
    for (verb, entity, id) in crate::grammar::command_paths() {
        let schema: Value = serde_json::from_str(
            crate::schema::find(id).ok_or_else(|| io::Error::other("query schema"))?,
        )?;
        if !super::typed::catalog::available(verb, entity, &schema, &excluded, true) {
            continue;
        }
        let source_exclusions = super::typed::catalog::capability_excluded(&schema, &excluded);
        let sources = crate::help::coverage::command_sources(verb, entity, &schema)
            .into_iter()
            .filter(|word| {
                !word
                    .parse::<Source>()
                    .is_ok_and(|source| source_exclusions.contains(&source))
            })
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(format!("{verb} {entity}: {sources}"));
    }
    Ok(rows.join("\n"))
}

fn next_allowed(words: &[String], cx: Context<'_>) -> bool {
    let args = words.iter().map(OsString::from).collect::<Vec<_>>();
    crate::grammar::parse(cx.root, &args).is_ok_and(|mut parsed| {
        parsed.globals.inherit(cx.globals).is_ok()
            && parsed.action.is_some()
            && crate::command::mcp_policy::refusal(&parsed, true).is_none()
            && parsed.action.as_ref().is_none_or(|action| match action {
                biomcp_query::Action::Run { request, .. } => super::typed::catalog::excluded(cx)
                    .and_then(|excluded| super::typed::catalog::request_allowed(request, &excluded))
                    .unwrap_or(false),
                _ => true,
            })
    })
}

fn offered(
    path: &[String],
    cx: Context<'_>,
    excluded: &std::collections::BTreeSet<Source>,
) -> io::Result<bool> {
    if path.is_empty() {
        return Ok(true);
    }
    let mut known = false;
    for (verb, entity, id) in crate::grammar::command_paths() {
        if ![verb.to_string(), entity.to_string()].starts_with(path) {
            continue;
        }
        known = true;
        let schema: Value = serde_json::from_str(
            crate::schema::find(id).ok_or_else(|| io::Error::other("query schema"))?,
        )?;
        if super::typed::catalog::available(verb, entity, &schema, excluded, true) {
            return Ok(true);
        }
    }
    Ok(!known && next_allowed(path, cx))
}

fn reason(
    source: Source,
    cx: Context<'_>,
    required: &std::collections::BTreeSet<Source>,
) -> io::Result<String> {
    let operator = crate::config::excluded_sources(cx.globals, cx.settings)
        .map_err(|e| io::Error::other(e.0))?;
    // A source the use limit keeps off stays off whatever the list says, so say that first.
    let use_limit = crate::command::hosted_use::disabled(cx.declared_use, cx.secrets);
    let limited = use_limit.contains(&source)
        || biomcp_query::sources::components().any(|component| {
            component.source == source
                && component
                    .native_source
                    .is_some_and(|native| required.contains(&native) && use_limit.contains(&native))
        });
    if !operator.contains(&source)
        && !limited
        && cx.excluded.is_some_and(|excluded| {
            excluded.contains(&source)
                || biomcp_query::sources::components().any(|component| {
                    component.source == source
                        && component.native_source.is_some_and(|native| {
                            required.contains(&native) && excluded.contains(&native)
                        })
                })
        })
    {
        return Ok(
            "This request's source list leaves out this source or a required component. \
                   Change the source list in your account settings to include it."
                .to_owned(),
        );
    }
    let component_disabled = |hosted| {
        cx.fetch.as_ref().is_ok_and(|fetch| {
            let effective = if hosted {
                crate::command::hosted_use::restrict(
                    fetch.restrict_sources(Default::default()),
                    Some(cx),
                )
            } else {
                fetch.restrict_sources(Default::default())
            };
            biomcp_query::sources::components().any(|component| {
                component.source == source
                    && component
                        .native_source
                        .is_some_and(|source| required.contains(&source))
                    && component.disabled(&effective)
            })
        })
    };
    let explanation = if operator.contains(&source) || component_disabled(false) {
        "The server operator disabled this source or a required component.".to_owned()
    } else if limited || component_disabled(true) {
        let ceiling =
            crate::config::hosted_max_use(cx.settings).map_err(|e| io::Error::other(e.0))?;
        let effective = cx
            .declared_use
            .unwrap_or(crate::command::DeclaredUse::Academic);
        format!(
            "This source or a required component is disabled by the requested hosted use policy (permitted use: {}). The server ceiling permits {} use.",
            effective.allowed(),
            ceiling.allowed()
        )
    } else {
        "This source or a required component is disabled for this request.".to_owned()
    };
    Ok(format!(
        "{explanation} A self-hosted install can enable it subject to the publisher's terms."
    ))
}

fn sources(cx: Context<'_>) -> io::Result<String> {
    let excluded = super::typed::catalog::excluded(cx)?;
    let mut rows = Vec::new();
    for spec in biomcp_query::sources::sources() {
        let source = spec.source;
        rows.push(if excluded.contains(&source) {
            format!("{source}: {}", reason(source, cx, &Default::default())?)
        } else { format!("{source}: served by this server; publisher terms and credential requirements apply.") });
    }
    Ok(rows.join("\n"))
}
