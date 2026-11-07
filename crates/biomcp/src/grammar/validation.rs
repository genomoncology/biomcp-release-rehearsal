//! Derive accepted values and forms from the command's owning schema.
use biomcp_types::{QueryInvalid, UsageArguments};
use serde_json::Value;

pub(super) fn schema(verb: &str, entity: &str) -> Option<Value> {
    let id = super::command_paths()
        .iter()
        .find(|(v, e, _)| *v == verb && *e == entity)?
        .2;
    serde_json::from_str(crate::schema::find(id)?).ok()
}

pub(super) fn checked(verb: &str, entity: &str, cause: &QueryInvalid) -> Option<UsageArguments> {
    let schema = schema(verb, entity)?;
    let (name, property) = cause.arguments().iter().find_map(|name| {
        schema
            .get("properties")?
            .get(name.as_ref())
            .map(|property| (name.as_ref(), property))
    })?;
    let argument = if property.get("x-positional").is_some() {
        name.into()
    } else {
        format!("--{}", name.replace('_', "-"))
    };
    Some(UsageArguments {
        argument: Some(argument.into()),
        message: cause.message(),
        valid_values: choices(name).or_else(|| accepted(&schema, property)),
        next: vec![help([verb, entity])],
    })
}

/// Name the real choices of a free-text argument whose schema carries only a type description.
fn choices(name: &str) -> Option<Box<[biodata::Text]>> {
    let values: &[&str] = match name {
        "label_section" => &biomcp_types::OPENFDA_LABEL_HEADINGS,
        _ => return None,
    };
    values
        .iter()
        .map(|value| biodata::Text::new(value).ok())
        .collect::<Option<Vec<_>>>()
        .map(Into::into)
}

pub(super) fn help<'a>(path: impl IntoIterator<Item = &'a str>) -> biomcp_types::Step {
    biomcp_types::Step {
        kind: biomcp_types::StepKind::Configure,
        args: std::iter::once("help")
            .chain(path)
            .map(Box::from)
            .collect::<Vec<_>>()
            .into(),
    }
}

pub(super) fn resolve<'a>(schema: &'a Value, property: &'a Value) -> &'a Value {
    if let Some(reference) = property.get("$ref").and_then(Value::as_str) {
        return schema
            .pointer(reference.trim_start_matches('#'))
            .unwrap_or(property);
    }
    if let Some(choices) = property.get("anyOf").and_then(Value::as_array)
        && let Some(choice) = choices
            .iter()
            .find(|choice| choice.get("type").and_then(Value::as_str) != Some("null"))
    {
        return resolve(schema, choice);
    }
    if let Some(items) = property.get("items") {
        return resolve(schema, items);
    }
    property
}

pub(super) fn accepted(schema: &Value, property: &Value) -> Option<Box<[biodata::Text]>> {
    let value = resolve(schema, property);
    let forms = if let Some(values) = value.get("enum").and_then(Value::as_array) {
        values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect()
    } else if super::property_type(value) == Some("integer") {
        let mut form = String::from("integer");
        for (key, operator) in [("minimum", ">="), ("maximum", "<=")] {
            if let Some(bound) = value.get(key) {
                form.push_str(&format!(" {operator} {bound}"));
            }
        }
        vec![form]
    } else {
        let description = value
            .get("description")
            .or_else(|| property.get("description"))
            .and_then(Value::as_str)
            .or_else(|| super::property_type(value))?;
        let mut form = description.split_whitespace().collect::<Vec<_>>().join(" ");
        for (key, label) in [
            ("minimum", "Minimum"),
            ("maximum", "Maximum"),
            ("minLength", "Minimum length"),
            ("maxLength", "Maximum length"),
        ] {
            if let Some(bound) = value.get(key) {
                form.push_str(&format!(" {label}: {bound}."));
            }
        }
        vec![form]
    };
    forms
        .into_iter()
        .map(|form| biodata::Text::new(&form).ok())
        .collect::<Option<Vec<_>>>()
        .map(Into::into)
}
