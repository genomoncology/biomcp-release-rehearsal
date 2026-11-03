//! Construct query arguments from their owning schemas.
use super::{gwas, properties, property_type};
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
            argument.action(ArgAction::SetTrue)
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
    if schema.get("$id").and_then(Value::as_str) == Some("biomcp.article-search/3") {
        command = command.arg(
            Arg::new("query_alias")
                .long("query")
                .short('q')
                .help("Alias for positional article text.")
                .conflicts_with("text"),
        );
    }
    if matches!(name, "cell-line" | "author") && properties(schema)?.contains_key("query") {
        command = command.mut_arg("limit", |arg| arg.short('l'));
    }
    Ok(if name == "gwas" {
        gwas::aliases(command, schema)
    } else if schema.get("$id").and_then(Value::as_str).is_some_and(|id| {
        id.starts_with("biomcp.pathway-")
            || id.starts_with("biomcp.author-")
            || id == "biomcp.phenotype-search-query/1"
    }) && properties(schema)?.contains_key("limit")
    {
        command.mut_arg("limit", |argument| argument.short('l'))
    } else {
        command
    })
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
