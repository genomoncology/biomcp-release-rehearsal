//! Compare only the named volatile fields approved in PM ruling m25543.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// Store::publish and publish_source_pool retain a one-day immutable pool.
const POOL_TTL_SECONDS: u64 = 86_400;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReviewedFields {
    pointers: Vec<String>,
    #[serde(rename = "capture_sha256")]
    _capture_sha256: String,
}

fn pointers(case: &str, step: usize) -> Result<Option<Vec<String>>, String> {
    let inventory: BTreeMap<String, ReviewedFields> =
        serde_json::from_str(include_str!("typed_volatile_cases.json"))
            .map_err(|error| error.to_string())?;
    Ok(inventory
        .get(&format!("{case} step {step}"))
        .map(|entry| entry.pointers.clone()))
}

pub(super) fn expected(
    case: &str,
    step: usize,
    before: &Value,
    after: &Value,
) -> Result<Option<Value>, String> {
    let Some(paths) = pointers(case, step)? else {
        return Ok(None);
    };
    project(&paths, before, after).map(Some)
}

fn project(paths: &[String], before: &Value, after: &Value) -> Result<Value, String> {
    let mut expected = before.clone();
    let mut handles = BTreeMap::<String, String>::new();
    for path in paths {
        let old = before
            .pointer(path)
            .ok_or_else(|| format!("missing approved field {path}"))?;
        let new = after
            .pointer(path)
            .ok_or_else(|| format!("missing approved field {path}"))?;
        if path.ends_with("/created")
            || path.ends_with("/expires")
            || path == "/result/snapshot/acquired"
        {
            if old.as_u64().is_none() || new.as_u64().is_none() {
                return Err(format!("approved local timestamp changed type at {path}"));
            }
        } else {
            let old = handle(old, path)?;
            let new = handle(new, path)?;
            if handles.get(old).is_some_and(|prior| prior != new) {
                return Err(format!(
                    "one captured handle has inconsistent references at {path}"
                ));
            }
            handles.insert(old.to_owned(), new.to_owned());
        }
        *expected
            .pointer_mut(path)
            .ok_or("approved field missing from expected reply")? = new.clone();
    }
    let distinct: BTreeSet<_> = handles.values().collect();
    if distinct.len() != handles.len() {
        return Err("distinct captured handles collapsed into one handle".into());
    }
    references(before, after, &handles, "")?;
    pool_ttl(before, &handles.keys().cloned().collect())?;
    pool_ttl(after, &handles.values().cloned().collect())?;
    Ok(expected)
}

fn handle<'a>(value: &'a Value, path: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .filter(|value| {
            value.len() == 32
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        })
        .ok_or_else(|| format!("approved handle changed its declared hexadecimal form at {path}"))
}

fn references(
    before: &Value,
    after: &Value,
    handles: &BTreeMap<String, String>,
    path: &str,
) -> Result<(), String> {
    match before {
        Value::String(value) => {
            if let Some(expected) = handles.get(value)
                && after.pointer(path).and_then(Value::as_str) != Some(expected.as_str())
            {
                return Err(format!(
                    "captured handle reference is inconsistent at {path}"
                ));
            }
        }
        Value::Object(fields) => {
            for (key, value) in fields {
                let key = key.replace('~', "~0").replace('/', "~1");
                references(value, after, handles, &format!("{path}/{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                references(value, after, handles, &format!("{path}/{index}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn pool_ttl(value: &Value, handles: &BTreeSet<String>) -> Result<(), String> {
    match value {
        Value::Object(fields) => {
            if fields
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| handles.contains(id))
                && (fields.contains_key("created") || fields.contains_key("expires"))
            {
                let created = fields
                    .get("created")
                    .and_then(Value::as_u64)
                    .ok_or("pool created time")?;
                let expires = fields
                    .get("expires")
                    .and_then(Value::as_u64)
                    .ok_or("pool expiry time")?;
                if created.checked_add(POOL_TTL_SECONDS) != Some(expires) {
                    return Err(
                        "captured pool expiry does not equal creation plus the declared TTL".into(),
                    );
                }
            }
            for value in fields.values() {
                pool_ttl(value, handles)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                pool_ttl(value, handles)?;
            }
        }
        _ => {}
    }
    Ok(())
}

tool_only! {
pub(super) fn clock_cases() -> Result<Vec<String>, String> {
    let inventory: BTreeMap<String, ReviewedFields> =
        serde_json::from_str(include_str!("typed_clock_cases.json"))
            .map_err(|error| error.to_string())?;
    Ok(inventory.into_keys().collect())
}

pub(super) fn clock(case: &str, step: usize, before: &str, after: &str) -> Result<(), String> {
    let inventory: BTreeMap<String, ReviewedFields> =
        serde_json::from_str(include_str!("typed_clock_cases.json"))
            .map_err(|error| error.to_string())?;
    let key = format!("{case} step {step}");
    let extra = &inventory
        .get(&key)
        .ok_or("unlisted delayed clock probe")?
        .pointers;
    if key == "study/examples.yaml step 5" {
        if extra != &["/result/snapshot/acquired"] {
            return Err("unexpected study acquisition control path".into());
        }
        return study_clock(before, after);
    }
    let mut fields = pointers(case, step)?.unwrap_or_default();
    fields.extend(extra.iter().cloned());
    let old: Value = serde_json::from_str(before).map_err(|error| error.to_string())?;
    let new: Value = serde_json::from_str(after).map_err(|error| error.to_string())?;
    for pointer in extra {
        let path = format!("/result/structuredContent{pointer}");
        if old.pointer(&path) == new.pointer(&path) {
            return Err(format!(
                "delayed CURRENT/CURRENT clock did not differ at {pointer}"
            ));
        }
    }
    if !raw_fields(&fields, before, after)? {
        return Err("delayed CURRENT/CURRENT differs outside the named pool fields".into());
    }
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "TYPED_CLOCK_CONTROL case={key} exact_clock_paths={extra:?} other_fields=strict"
    )
    .map_err(|error| error.to_string())
}

// Diagnosis only: this never grants an allowance to the full raw comparison.
fn study_clock(before: &str, after: &str) -> Result<(), String> {
    let mut old: Value = serde_json::from_str(before).map_err(|error| error.to_string())?;
    let new: Value = serde_json::from_str(after).map_err(|error| error.to_string())?;
    let serialized_old = old.to_string();
    if serialized_old != before {
        return Err("study CURRENT raw frame changed canonical form".into());
    }
    let path = "/result/structuredContent/result/snapshot/acquired";
    let first = old.pointer(path).and_then(Value::as_u64).ok_or("study acquisition timestamp")?;
    let last = new.pointer(path).and_then(Value::as_u64).ok_or("study acquisition control timestamp")?;
    if last <= first {
        return Err("delayed study CURRENT/CURRENT acquisition clock did not advance".into());
    }
    let body = old.pointer("/result/structuredContent").ok_or("study body")?;
    let serialized_body = body.to_string();
    if old.pointer("/result/content/0/text").and_then(Value::as_str) != Some(serialized_body.as_str()) {
        return Err("study CURRENT text is not its exact serialized body".into());
    }
    *old.pointer_mut(path).ok_or("study clock field")? = Value::from(last);
    let body = old.pointer("/result/structuredContent").ok_or("study updated body")?.to_string();
    *old.pointer_mut("/result/content/0/text").ok_or("study text")? = Value::String(body);
    let expected = old.to_string();
    if expected != after {
        return Err("study delayed CURRENT/CURRENT differs outside acquisition clock".into());
    }
    use std::io::Write;
    writeln!(std::io::stdout(), "TYPED_CLOCK_CONTROL case=study/examples.yaml step 5 \
         exact_clock_paths=[/result/snapshot/acquired] other_fields=strict diagnostic_only=true")
        .map_err(|error| error.to_string())
}


}

pub(super) fn raw(case: &str, step: usize, before: &str, after: &str) -> Result<bool, String> {
    let Some(paths) = pointers(case, step)? else {
        return Ok(false);
    };
    raw_fields(&paths, before, after)
}

fn raw_fields(paths: &[String], before: &str, after: &str) -> Result<bool, String> {
    let old: Value = serde_json::from_str(before).map_err(|error| error.to_string())?;
    let new: Value = serde_json::from_str(after).map_err(|error| error.to_string())?;
    let serialized_old = old.to_string();
    if serialized_old != before {
        return Err("single raw reply is not in its existing canonical wire form".into());
    }
    let old_body = old
        .pointer("/result/structuredContent")
        .ok_or("single structured reply")?;
    let new_body = new
        .pointer("/result/structuredContent")
        .ok_or("single structured reply")?;
    let expected_body = project(paths, old_body, new_body)?;
    let mut expected = old.clone();
    if let Some(text) = old
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
    {
        let serialized_body = old_body.to_string();
        if text != serialized_body {
            return Err("single text is not the exact existing serialized structured reply".into());
        }
        *expected
            .pointer_mut("/result/content/0/text")
            .ok_or("single text")? = Value::String(expected_body.to_string());
    }
    *expected
        .pointer_mut("/result/structuredContent")
        .ok_or("single structured reply")? = expected_body;
    let serialized_expected = expected.to_string();
    Ok(serialized_expected == after)
}
