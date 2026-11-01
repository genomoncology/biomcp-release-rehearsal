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

fn invalid_argument<'a>(command: &'a Command, error: &clap::Error) -> Option<&'a clap::Arg> {
    let spellings: Vec<_> = match error.get(ContextKind::InvalidArg)? {
        ContextValue::String(value) => vec![value.as_str()],
        ContextValue::Strings(values) => values.iter().map(String::as_str).collect(),
        _ => return None,
    };
    spellings.iter().find_map(|spelling| {
        command
            .get_arguments()
            .find(|argument| argument.to_string() == *spelling)
    })
}

fn parser_usage(command: &Command, path: &[&str], error: &clap::Error) -> Option<UsageArguments> {
    let (argument, valid_values) = if let Some(argument) = invalid_argument(command, error) {
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

/// Preserve parser-owned identities and command guidance from the first parse.
pub(super) fn parse_invalid(root: &Command, error: &clap::Error) -> super::ParseInvalid {
    let (command, path) = command_context(root, error);
    if let Some(arguments) = parser_usage(command, &path, error) {
        return super::ParseInvalid {
            argument: None,
            usage: Some(UsageError::Arguments(arguments)),
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
    }
}

/// Carry checked constructor guidance without reading caller arguments again.
pub(super) fn checked_invalid(
    verb: &str,
    entity: &str,
    cause: &biomcp_types::QueryInvalid,
) -> super::ParseInvalid {
    let usage = super::validation::checked(verb, entity, cause)
        .map(UsageError::Arguments)
        .or_else(|| cause.message().map(UsageError::message));
    super::ParseInvalid {
        argument: None,
        usage,
    }
}
