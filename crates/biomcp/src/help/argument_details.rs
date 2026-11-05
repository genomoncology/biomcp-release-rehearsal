//! Combine schema argument facts, recorded examples and registry source coverage.
use biomcp_types::{HelpArgument, HelpCommand};

pub(super) fn complete(commands: &mut [HelpCommand]) {
    for command in commands {
        let sources = command
            .command
            .get(..2)
            .and_then(|path| {
                let [verb, entity] = path else {
                    return None;
                };
                crate::grammar::command_paths()
                    .iter()
                    .find(|(verb, entity, _)| path == [*verb, *entity])
                    .and_then(|(_, _, id)| crate::schema::find(id))
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
                    .map(|schema| {
                        super::coverage::command_sources(verb, entity, &schema)
                            .into_iter()
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
            })
            .unwrap_or_default();
        for argument in &mut command.arguments {
            if !argument.name.starts_with("--") {
                continue;
            }
            let sample = example(argument, &command.examples);
            let scope = if sources.is_empty() {
                "Local command option; independent of source selection.".into()
            } else {
                format!(
                    "Command sources: {sources}; source restrictions in the field description apply."
                )
            };
            let sample = if argument.description.contains("Example:") {
                String::new()
            } else {
                format!(" Example: {sample}.")
            };
            let form = if argument.value_type == "array" {
                "repeatable values (repeat the flag)"
            } else {
                &argument.value_type
            };
            argument
                .description
                .push_str(&format!(" Value form: {form}.{sample} {scope}"));
        }
    }
}
fn example(argument: &HelpArgument, examples: &[Vec<String>]) -> String {
    for example in examples {
        for (index, word) in example.iter().enumerate() {
            if word == &argument.name {
                let value = example
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"));
                return value.map_or_else(
                    || word.clone(),
                    |value| {
                        format!(
                            "{word} {}",
                            shlex::try_quote(value)
                                .map_or_else(|_| "<value>".into(), |value| value.into_owned())
                        )
                    },
                );
            }
            if word.starts_with(&format!("{}=", argument.name)) {
                return word.clone();
            }
        }
    }
    if argument.value_type == "boolean" {
        return argument.name.clone();
    }
    let value = argument
        .choices
        .first()
        .cloned()
        .or_else(|| argument.default.clone())
        .or_else(|| {
            (argument.value_type == "integer" || argument.value_type == "number")
                .then(|| argument.minimum.unwrap_or(0).to_string())
        });
    let Some(value) = value else {
        return format!(
            "{} <{}> (syntax; replace the placeholder with your retained value)",
            argument.name,
            argument.name.trim_start_matches("--")
        );
    };
    format!(
        "{} {}",
        argument.name,
        shlex::try_quote(&value).map_or_else(|_| "<value>".into(), |value| value.into_owned())
    )
}
