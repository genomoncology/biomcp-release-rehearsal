//! Recover parser-owned argument identities without copying rejected values.
use biomcp_types::{UsageArguments, UsageError};
use clap::{
    Command,
    error::{ContextKind, ContextValue, ErrorKind},
};

// The parser's usage context names its own command, including when parsing stopped
// before matches could be returned. Never recover that path from caller argv.
fn command_context<'a>(root: &'a Command, error: &clap::Error) -> (&'a Command, Vec<&'a str>) {
    let usage = error
        .get(ContextKind::Usage)
        .map(ToString::to_string)
        .unwrap_or_default();
    let mut words = usage
        .split_whitespace()
        .skip_while(|word| *word != root.get_name());
    words.next();
    let mut command = root;
    let mut path = Vec::new();
    for word in words {
        let Some(child) = command.find_subcommand(word) else {
            break;
        };
        path.push(child.get_name());
        command = child;
    }
    (command, path)
}

fn invalid_argument<'a>(
    command: &'a Command,
    path: &[&str],
    error: &clap::Error,
) -> Option<&'a clap::Arg> {
    let spellings: Vec<_> = match error.get(ContextKind::InvalidArg)? {
        ContextValue::String(value) => vec![value.as_str()],
        ContextValue::Strings(values) => values.iter().map(String::as_str).collect(),
        _ => return None,
    };
    spellings
        .iter()
        .find_map(|spelling| {
            command
                .get_arguments()
                .find(|argument| argument.to_string() == *spelling)
        })
        .or_else(|| excess_positional(command, path, error))
}

fn excess_positional<'a>(
    command: &'a Command,
    path: &[&str],
    error: &clap::Error,
) -> Option<&'a clap::Arg> {
    if error.kind() != ErrorKind::UnknownArgument {
        return None;
    }
    let ContextValue::String(spelling) = error.get(ContextKind::InvalidArg)? else {
        return None;
    };
    let [verb, entity] = path else { return None };
    let schema = super::validation::schema(verb, entity)?;
    let argument = command
        .get_positionals()
        .max_by_key(|arg| arg.get_index())?;
    let property = schema.get("properties")?.get(argument.get_id().as_str())?;
    super::validation::resolve(&schema, property)
        .get("enum")?
        .as_array()?
        .iter()
        .any(|value| value.as_str() == Some(spelling))
        .then_some(argument)
}

fn parser_usage(command: &Command, path: &[&str], error: &clap::Error) -> Option<UsageArguments> {
    let (argument, valid_values) = if let Some(argument) = invalid_argument(command, path, error) {
        let identity = argument
            .get_long()
            .map_or_else(|| argument.get_id().to_string(), |name| format!("--{name}"));
        let schema = path
            .first()
            .zip(path.get(1))
            .and_then(|(verb, entity)| super::validation::schema(verb, entity));
        let values = schema
            .as_ref()
            .and_then(|schema| {
                super::validation::accepted(
                    schema,
                    schema.get("properties")?.get(argument.get_id().as_str())?,
                )
            })
            .or_else(|| {
                let values: Vec<_> = argument
                    .get_value_parser()
                    .possible_values()?
                    .map(|value| biodata::Text::new(value.get_name()).ok())
                    .collect::<Option<_>>()?;
                (!values.is_empty()).then(|| values.into())
            })
            .or_else(|| {
                let form = if argument.get_action().takes_values() {
                    "one value"
                } else {
                    "a flag without a value"
                };
                Some(vec![biodata::Text::new(form).ok()?].into())
            });
        (identity, values)
    } else if error.kind() == ErrorKind::UnknownArgument {
        let ContextValue::String(spelling) = error.get(ContextKind::InvalidArg)? else {
            return None;
        };
        // Only retain an option name supplied by the parser. Never echo values,
        // including an inline value or an unexpected positional operand.
        let spelling = spelling.split('=').next()?;
        if !spelling.starts_with('-')
            || !spelling
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return None;
        }
        let values = command
            .get_arguments()
            .filter_map(|argument| argument.get_long())
            .map(|name| biodata::Text::new(&format!("--{name}")).ok())
            .collect::<Option<Vec<_>>>()?;
        (spelling.to_owned(), Some(values.into()))
    } else {
        return None;
    };
    Some(UsageArguments {
        argument: Some(argument.into()),
        message: None,
        valid_values,
        next: vec![super::validation::help(path.iter().copied())],
    })
}

fn parser_correction(
    root: &Command,
    path: &[&str],
    error: &clap::Error,
    args: &[std::ffi::OsString],
) -> Option<biomcp_types::Step> {
    // A stopped value parse can omit later filters; never turn that partial input into a new query.
    if !matches!(
        error.kind(),
        ErrorKind::MissingRequiredArgument | ErrorKind::UnknownArgument
    ) {
        return None;
    }
    let [verb, entity] = path else { return None };
    let command = root.find_subcommand(verb)?.find_subcommand(entity)?;
    let argument = invalid_argument(command, path, error)?;
    let mut args = args.to_vec();
    if error.kind() == ErrorKind::UnknownArgument
        && let Some(ContextValue::String(spelling)) = error.get(ContextKind::InvalidArg)
        && !spelling.starts_with('-')
        && let Some(index) = args
            .iter()
            .rposition(|word| word.to_str() == Some(spelling))
    {
        args.remove(index);
    }
    let parser = if error.kind() == ErrorKind::MissingRequiredArgument {
        root.clone()
            .ignore_errors(true)
            .mut_subcommand(*verb, |command| {
                command
                    .ignore_errors(true)
                    .mut_subcommand(*entity, |command| command.ignore_errors(true))
            })
    } else {
        root.clone()
    };
    let matches = parser
        .try_get_matches_from(std::iter::once(std::ffi::OsString::from("biomcp")).chain(args))
        .ok()?;
    let matches = matches
        .subcommand_matches(verb)?
        .subcommand_matches(entity)?;
    let value = super::request_arguments(verb, entity, matches).ok()?;
    let schema = super::validation::schema(verb, entity)?;
    let declaration = biomcp_types::COMMANDS
        .iter()
        .find(|command| (command.verb, command.entity) == (*verb, *entity))?;
    super::recovery::corrected(
        declaration.via,
        (verb, entity),
        &schema,
        &value,
        &[argument.get_id().as_str()],
        false,
    )
}

/// Preserve parser-owned identities and command guidance from the first parse.
pub(super) fn parse_invalid(
    root: &Command,
    error: &clap::Error,
    args: &[std::ffi::OsString],
) -> super::ParseInvalid {
    let (command, path) = command_context(root, error);
    if let Some(mut arguments) = parser_usage(command, &path, error) {
        let registered_argument = invalid_argument(command, &path, error).is_some();
        if registered_argument {
            super::recovery::next(&mut arguments, parser_correction(root, &path, error, args));
        }
        return super::ParseInvalid {
            argument: None,
            usage: Some(UsageError::Arguments(arguments)),
            prefer_argument_error: registered_argument,
        };
    }
    let message = match path.as_slice() {
        ["search", "gwas"] => Some(biomcp_types::ErrorMessage::GwasSearchArguments),
        ["get", "gwas"] => Some(biomcp_types::ErrorMessage::GwasGetArguments),
        _ => None,
    };
    super::ParseInvalid {
        argument: None,
        usage: message.map(UsageError::message),
        prefer_argument_error: false,
    }
}

/// Carry checked constructor guidance without reading caller arguments again.
pub(super) fn checked_invalid(
    via: biomcp_types::Rewrite,
    verb: &str,
    entity: &str,
    value: &serde_json::Value,
    cause: &biomcp_types::QueryInvalid,
) -> super::ParseInvalid {
    let usage = super::validation::checked(verb, entity, cause)
        .map(|mut arguments| {
            let names: Vec<_> = cause.arguments().iter().map(AsRef::as_ref).collect();
            let corrected = super::recovery::trial_first_page(via, (verb, entity), value, cause)
                .or_else(|| {
                    super::validation::schema(verb, entity).and_then(|schema| {
                        super::recovery::corrected(
                            via,
                            (verb, entity),
                            &schema,
                            value,
                            &names,
                            false,
                        )
                    })
                });
            super::recovery::next(&mut arguments, corrected);
            arguments
        })
        .map(UsageError::Arguments)
        .or_else(|| cause.message().map(UsageError::message));
    super::ParseInvalid {
        argument: None,
        usage,
        // A rejected value may still be an explicitly retired spelling.
        prefer_argument_error: false,
    }
}

/// Derive source/filter compatibility from the same checked constructors as execution.
pub(super) fn source_filter(
    via: biomcp_types::Rewrite,
    verb: &str,
    entity: &str,
    value: &serde_json::Value,
    cause: &biomcp_types::QueryInvalid,
) -> Option<super::ParseInvalid> {
    if !cause
        .arguments()
        .iter()
        .any(|name| name.as_ref() == "source")
    {
        return None;
    }
    let schema = super::validation::schema(verb, entity)?;
    let properties = schema.get("properties")?;
    let selected = selected_sources(value, &schema)?;
    for name in cause
        .arguments()
        .iter()
        .map(AsRef::as_ref)
        .filter(|name| *name != "source")
    {
        if properties.get(name).is_none_or(|property| {
            property.get("x-positional").is_some() && property.get("x-section-rule").is_none()
        }) {
            continue;
        }
        let Some(filter_value) = value.get(name) else {
            continue;
        };
        let valid = accepting_sources(via, (verb, entity), value, &schema, name)?;
        if valid.is_empty()
            || selected
                .iter()
                .all(|source| valid.iter().any(|candidate| candidate.as_str() == *source))
        {
            continue;
        }
        let equivalent = super::metadata::FILTER_EQUIVALENTS
            .iter()
            .find(|(v, e, s, from, _)| {
                (*v, *e, *from) == (verb, entity, name) && selected.as_slice() == [*s]
            });
        let next = if let Some((_, _, _, _, replacement)) = equivalent {
            let mut corrected = value.clone();
            corrected.as_object_mut()?.remove(name);
            corrected
                .as_object_mut()?
                .insert((*replacement).into(), filter_value.clone());
            replacement_step(via, verb, entity, &schema, &corrected)
                .unwrap_or_else(|| super::validation::help([verb, entity]))
        } else {
            super::recovery::corrected(via, (verb, entity), &schema, value, &[name], true)
                .unwrap_or_else(|| super::validation::help([verb, entity]))
        };
        return Some(super::ParseInvalid {
            argument: None,
            usage: Some(UsageError::Arguments(UsageArguments {
                argument: Some(format!("--{}", name.replace('_', "-")).into()),
                message: cause
                    .message()
                    .or(Some(biomcp_types::ErrorMessage::UnsupportedFilter)),
                valid_values: Some(valid.into()),
                next: super::recovery::guidance(next, [verb, entity]),
            })),
            prefer_argument_error: true,
        });
    }
    None
}

fn selected_sources<'a>(
    value: &'a serde_json::Value,
    schema: &'a serde_json::Value,
) -> Option<Vec<&'a str>> {
    let source = value
        .get("source")
        .or_else(|| schema.get("properties")?.get("source")?.get("default"))?;
    let values = source
        .as_array()
        .map_or_else(|| std::slice::from_ref(source), Vec::as_slice);
    values.iter().map(serde_json::Value::as_str).collect()
}

fn accepting_sources(
    via: biomcp_types::Rewrite,
    path: (&str, &str),
    value: &serde_json::Value,
    schema: &serde_json::Value,
    name: &str,
) -> Option<Vec<biodata::Text>> {
    let sources = super::validation::accepted(schema, schema.get("properties")?.get("source")?)?;
    Some(
        sources
            .iter()
            .filter(|source| {
                accepts_filter(via, path, value, schema, name, source.as_str()).unwrap_or(false)
            })
            .cloned()
            .collect(),
    )
}

// Remove only other filters a candidate source rejects. Keep positional operands
// and dependent filters, so the owning constructor remains the compatibility rule.
fn accepts_filter(
    via: biomcp_types::Rewrite,
    path: (&str, &str),
    value: &serde_json::Value,
    schema: &serde_json::Value,
    name: &str,
    source: &str,
) -> Option<bool> {
    let mut isolated = value.as_object()?.clone();
    let source = if value.get("source").is_some_and(serde_json::Value::is_array)
        || super::property_type(schema.get("properties")?.get("source")?) == Some("array")
    {
        serde_json::Value::Array(vec![source.into()])
    } else {
        source.into()
    };
    isolated.insert("source".into(), source);
    loop {
        let Err(cause) = super::rewrite::request(
            via,
            path.0,
            path.1,
            serde_json::Value::Object(isolated.clone()),
        ) else {
            return Some(true);
        };
        if !cause
            .arguments()
            .iter()
            .any(|name| name.as_ref() == "source")
        {
            return Some(false);
        }
        let mut removed = false;
        for other in cause.arguments().iter().map(AsRef::as_ref) {
            if other == name
                || other == "source"
                || schema
                    .get("properties")?
                    .get(other)
                    .is_none_or(|property| property.get("x-positional").is_some())
                || schema
                    .get("required")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|required| {
                        required.iter().any(|value| value.as_str() == Some(other))
                    })
            {
                continue;
            }
            removed |= isolated.remove(other).is_some();
        }
        if !removed {
            return Some(false);
        }
    }
}

fn replacement_step(
    via: biomcp_types::Rewrite,
    verb: &str,
    entity: &str,
    schema: &serde_json::Value,
    value: &serde_json::Value,
) -> Option<biomcp_types::Step> {
    super::rewrite::request(via, verb, entity, value.clone()).ok()?;
    let mut args: Vec<Box<str>> = vec![verb.into(), entity.into()];
    let properties = schema.get("properties")?.as_object()?;
    for (name, property) in properties {
        if property.get("x-positional").is_some()
            && let Some(value) = value.get(name).and_then(serde_json::Value::as_str)
        {
            args.push(value.into());
        }
    }
    for (name, value) in value.as_object()? {
        if properties.get(name)?.get("x-positional").is_some() {
            continue;
        }
        let values = value
            .as_array()
            .map_or_else(|| vec![value], |values| values.iter().collect());
        for value in values {
            args.push(format!("--{}", name.replace('_', "-")).into());
            args.push(
                value
                    .as_str()
                    .map_or_else(|| value.to_string(), str::to_owned)
                    .into(),
            );
        }
    }
    Some(biomcp_types::Step {
        kind: biomcp_types::StepKind::Configure,
        args: args.into(),
    })
}
