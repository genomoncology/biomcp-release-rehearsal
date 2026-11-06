//! Render schema-fed command arguments for adapters and migration classification.
use serde_json::Value;
use std::ffi::OsString;

fn words(value: &Value) -> Vec<String> {
    match value {
        Value::String(value) => vec![value.clone()],
        Value::Array(values) => values.iter().flat_map(words).collect(),
        _ => vec![value.to_string()],
    }
}
pub(crate) fn cli(
    root: &clap::Command,
    verb: &str,
    entity: &str,
    schema: &Value,
    values: &Value,
) -> Option<Vec<OsString>> {
    let command = root.find_subcommand(verb)?.find_subcommand(entity)?;
    let mut args = vec![verb.into(), entity.into()];
    // Parser indices own positional order; flags precede them to keep literal values safe.
    let mut positional = Vec::new();
    for (key, value) in values.as_object()? {
        if let Some(flags) = schema
            .get("properties")?
            .get(key)
            .and_then(|property| property.get("x-cli-flags"))
            .and_then(Value::as_object)
        {
            let (flag, _) = flags.iter().find(|(_, selected)| *selected == value)?;
            args.push(format!("--{flag}").into());
            continue;
        }
        let argument = command
            .get_arguments()
            .find(|arg| arg.get_id().as_str() == key)?;
        if let Some(index) = argument.get_index() {
            positional.push((index, words(value)));
            continue;
        }
        let flag = argument.get_long()?;
        if let Some(boolean) = value.as_bool()
            && matches!(
                argument.get_action(),
                clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
            )
        {
            if argument.is_require_equals_set() {
                args.push(format!("--{flag}={boolean}").into());
            } else if boolean {
                args.push(format!("--{flag}").into());
            }
            continue;
        }
        let values = words(value);
        if schema
            .get("properties")
            .and_then(|fields| fields.get(key))
            .and_then(|field| field.get("x-comma-separated"))
            == Some(&Value::Bool(true))
        {
            args.push(format!("--{flag}={}", values.join(",")).into());
        } else {
            for value in values {
                args.push(format!("--{flag}={value}").into());
            }
        }
    }
    positional.sort_by_key(|(index, _)| *index);
    if positional
        .iter()
        .flat_map(|(_, words)| words)
        .any(|word| word.starts_with('-'))
    {
        args.push("--".into());
    }
    args.extend(
        positional
            .into_iter()
            .flat_map(|(_, values)| values)
            .map(OsString::from),
    );
    Some(args)
}

/// Retirement checks see canonical choices; execution keeps the caller's full argv.
pub(crate) fn canonical(root: &clap::Command, args: &[OsString]) -> Option<Vec<OsString>> {
    let args = super::entry::arguments(args, root);
    let args = super::arguments::boolean_literals(&args, root);
    let matches = root
        .clone()
        .try_get_matches_from(std::iter::once(OsString::from("biomcp")).chain(args))
        .ok()?;
    let (verb, command) = matches.subcommand()?;
    let (entity, command) = command.subcommand()?;
    let schema = super::validation::schema(verb, entity)?;
    let values = super::request_arguments(verb, entity, command).ok()?;
    cli(root, verb, entity, &schema, &values)
}
