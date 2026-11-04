//! Validate schema metadata independently from compatibility assertions.
use serde_json::Value;
type Check<T> = Result<T, String>;

pub(super) fn validate(object: &serde_json::Map<String, Value>) -> Check<()> {
    validate_cli_flags(object.get("x-cli-flags"))?;
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
