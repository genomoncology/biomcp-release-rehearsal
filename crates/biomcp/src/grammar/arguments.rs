//! Construct query arguments from their owning schemas.
pub(crate) use super::process_arguments::{boolean_argument, boolean_value};
use super::{properties, property_type};
use clap::{Arg, ArgAction, Command, builder::TypedValueParser};
use serde_json::Value;

pub(super) fn command_for(
    name: &'static str,
    schema: &Value,
) -> Result<Command, super::ParseInvalid> {
    let mut command = Command::new(name)
        .about(
            schema
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        )
        .disable_help_flag(true)
        .disable_help_subcommand(true);
    for (name, property) in properties(schema)? {
        if let Some(flags) = property.get("x-cli-flags").and_then(Value::as_object) {
            for flag in flags.keys() {
                command = command.arg(
                    Arg::new(flag.clone())
                        .long(flag.clone())
                        .action(ArgAction::SetTrue)
                        .conflicts_with_all(flags.keys().filter(|other| *other != flag).cloned()),
                );
            }
            continue;
        }
        let mut argument = Arg::new(name.clone()).help(
            property
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        );
        if let Some(position) = property.get("x-positional").and_then(Value::as_u64) {
            argument = argument.index(position as usize + 1);
        } else {
            argument = argument.long(name.replace('_', "-"));
        }
        if schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|required| required.contains(&Value::String(name.clone())))
        {
            argument = argument.required(true);
        }
        if property_type(property) == Some("array") {
            argument = if property.get("x-comma-separated") == Some(&Value::Bool(true)) {
                argument
                    .num_args(1)
                    .value_delimiter(',')
                    .action(ArgAction::Append)
            } else if property.get("x-trailing-section").is_some() {
                argument.num_args(1).action(ArgAction::Append)
            } else {
                argument.num_args(1..).action(ArgAction::Append)
            };
        }
        command = command.arg(if property_type(property) == Some("boolean") {
            boolean_argument(argument)
        } else {
            value_argument(argument, property)
        });
    }
    aliases(name, schema, command)
}
fn aliases(
    name: &str,
    schema: &Value,
    mut command: Command,
) -> Result<Command, super::ParseInvalid> {
    for alias in super::query_aliases::declarations(name, schema) {
        if let Some(long) = alias.long {
            command = command.arg(
                Arg::new(alias.argument)
                    .long(long)
                    .short(alias.short)
                    .help("Alias for positional article text.")
                    .conflicts_with(alias.field),
            );
        } else {
            command = command.mut_arg(alias.argument, |argument| argument.short(alias.short));
        }
    }
    Ok(command)
}

fn value_argument(argument: Arg, property: &Value) -> Arg {
    argument
        .allow_negative_numbers(true)
        .value_parser(QueryValueParser {
            integer: property_type(property) == Some("integer"),
            minimum: property.get("minimum").and_then(Value::as_u64),
            maximum: property.get("maximum").and_then(Value::as_u64),
        })
}

#[derive(Clone)]
struct QueryValueParser {
    integer: bool,
    minimum: Option<u64>,
    maximum: Option<u64>,
}
impl TypedValueParser for QueryValueParser {
    type Value = String;
    fn parse_ref(
        &self,
        command: &Command,
        argument: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<String, clap::Error> {
        let value = clap::builder::StringValueParser::new().parse_ref(command, argument, value)?;
        if self.integer
            && !value.parse::<u64>().is_ok_and(|n| {
                self.minimum.is_none_or(|min| n >= min) && self.maximum.is_none_or(|max| n <= max)
            })
        {
            use clap::error::{ContextKind, ContextValue, ErrorKind};
            let mut error = clap::Error::new(ErrorKind::ValueValidation).with_cmd(command);
            if let Some(argument) = argument {
                error.insert(
                    ContextKind::InvalidArg,
                    ContextValue::String(argument.to_string()),
                );
            }
            error.insert(
                ContextKind::Usage,
                ContextValue::StyledStr(command.clone().render_usage()),
            );
            return Err(error);
        }
        Ok(value)
    }
}

/// Optional boolean literals are consumed only when they are exactly true or false.
pub(super) fn boolean_literals(
    args: &[std::ffi::OsString],
    root: &Command,
) -> Vec<std::ffi::OsString> {
    let mut result = Vec::new();
    let mut command = root;
    let mut words = args.iter().peekable();
    while let Some(word) = words.next() {
        let Some(text) = word.to_str() else {
            result.push(word.clone());
            continue;
        };
        if text == "--" {
            result.push(word.clone());
            result.extend(words.cloned());
            break;
        }
        if let Some(child) = command.find_subcommand(text) {
            command = child;
        }
        let argument =
            super::entry::argument(command, text).or_else(|| super::entry::argument(root, text));
        if let Some((argument, inline)) = argument {
            if !inline && boolean_value(argument, words.peek().copied()) {
                let value = words
                    .next()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                result.push(format!("{text}={value}").into());
                continue;
            }
            result.push(word.clone());
            if !inline
                && argument.get_action().takes_values()
                && !argument.is_require_equals_set()
                && let Some(value) = words.next()
            {
                result.push(value.clone());
            }
            continue;
        }
        result.push(word.clone());
    }
    result
}
