//! Validate schema metadata independently from compatibility assertions.
use serde_json::Value;
type Check<T> = Result<T, String>;

pub(super) fn validate(object: &serde_json::Map<String, Value>) -> Check<()> {
    validate_cli_flags(object.get("x-cli-flags"))?;
    validate_enum_scope(object)?;
    let source = |value: &Value| {
        value
            .as_str()
            .is_some_and(|word| word.parse::<biomcp_types::Source>().is_ok())
    };
    if object.get("x-required-component").is_some_and(|value| {
        !source(value)
            && !value
                .as_array()
                .is_some_and(|words| !words.is_empty() && words.iter().all(source))
    }) {
        return Err("invalid required-component annotation".into());
    }
    if object.get("x-required-components").is_some_and(|value| {
        !value.as_object().is_some_and(|map| {
            !map.is_empty()
                && map.iter().all(|(name, component)| {
                    name.parse::<biomcp_types::Source>().is_ok() && source(component)
                })
        })
    }) {
        return Err("invalid source-component annotation".into());
    }
    if object
        .get("x-identifier")
        .is_some_and(|value| value != &Value::Bool(true))
    {
        return Err("unsupported identifier annotation".into());
    }
    if object.get("x-extensible-enum").is_some_and(|value| {
        !value
            .as_array()
            .is_some_and(|values| !values.is_empty() && values.iter().all(Value::is_string))
    }) {
        return Err("invalid extensible enum annotation".into());
    }
    if object.get("x-reply-query-titles").is_some_and(|value| {
        !value.as_array().is_some_and(|titles| {
            !titles.is_empty()
                && titles
                    .iter()
                    .all(|title| title.as_str().is_some_and(|title| !title.is_empty()))
        })
    }) {
        return Err("invalid reply query title annotation".into());
    }
    if object
        .get("x-section-rule")
        .is_some_and(|value| value.as_str().is_none_or(str::is_empty))
    {
        return Err("unsupported section-rule annotation".into());
    }
    if object
        .get("x-trailing-section")
        .is_some_and(|value| value != &Value::Bool(true))
    {
        return Err("unsupported trailing-section annotation".into());
    }
    if object
        .get("x-comma-separated")
        .is_some_and(|v| v != &Value::Bool(true))
    {
        return Err("unsupported comma-separated annotation".into());
    }
    Ok(())
}

fn validate_cli_flags(flags: Option<&Value>) -> Check<()> {
    if let Some(flags) = flags {
        let flags = flags.as_object().ok_or("unsupported CLI flag annotation")?;
        if flags.is_empty()
            || flags.iter().any(|(name, value)| {
                !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                    || !name.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
                    || value.as_str().is_none_or(str::is_empty)
            })
        {
            return Err("unsupported CLI flag annotation".into());
        }
    }
    Ok(())
}

fn validate_enum_scope(object: &serde_json::Map<String, Value>) -> Check<()> {
    let Some(scope) = object.get("x-enum-by-field") else {
        return Ok(());
    };
    let scope = scope.as_object().ok_or("invalid enum field scope")?;
    let field = scope
        .get("field")
        .and_then(Value::as_str)
        .ok_or("enum scope field")?;
    let default = scope
        .get("default")
        .and_then(Value::as_str)
        .ok_or("enum scope default")?;
    let choices = scope
        .get("choices")
        .and_then(Value::as_object)
        .ok_or("enum scope choices")?;
    let values = object
        .get("enum")
        .and_then(Value::as_array)
        .ok_or("enum scope values")?;
    if scope.len() != 3
        || field.is_empty()
        || !choices.contains_key(default)
        || choices.iter().any(|(key, list)| {
            key.is_empty()
                || list.as_array().is_none_or(|list| {
                    list.is_empty() || list.iter().any(|v| !v.is_string() || !values.contains(v))
                })
        })
    {
        return Err("invalid enum field scope".into());
    }
    Ok(())
}
pub(super) fn required_component_annotations_are_metadata_with_checked_sources() -> Check<()> {
    use serde_json::json;
    let old = json!({"$id":"biodata.example/1", "type":"object", "properties":{"name":{"type":"string"}},
        "required":["name"], "additionalProperties":false});
    for annotation in [
        json!({"x-required-component":"meddra"}),
        json!({"x-required-component":["fda-pma","fda-510k"]}),
        json!({"x-required-components":{"maude":"gmdn"}}),
    ] {
        let new = annotated(&old, &annotation)?;
        assert!(
            super::compatibility::check(&old, &new).is_ok(),
            "valid component metadata must preserve compatibility: {annotation}"
        );
    }
    for annotation in [
        json!({"x-required-component":null}),
        json!({"x-required-component":"unknown-component"}),
        json!({"x-required-component":[]}),
        json!({"x-required-component":["fda-pma","unknown-component"]}),
        json!({"x-required-components":[]}),
        json!({"x-required-components":{}}),
        json!({"x-required-components":{"maude":false}}),
        json!({"x-required-components":{"unknown":"gmdn"}}),
    ] {
        let new = annotated(&old, &annotation)?;
        assert!(
            super::compatibility::check(&old, &new).is_err(),
            "invalid component metadata must fail compatibility: {annotation}"
        );
    }
    Ok(())
}

fn annotated(old: &Value, annotation: &Value) -> Check<Value> {
    let mut new = old.clone();
    new.pointer_mut("/properties/name")
        .and_then(Value::as_object_mut)
        .ok_or("schema fixture needs an object name property")?
        .extend(
            annotation
                .as_object()
                .ok_or("annotation fixture must be an object")?
                .clone(),
        );
    Ok(new)
}
