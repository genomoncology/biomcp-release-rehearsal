//! Preserve literal source rows while rejecting one malformed optional field.
use super::response_cases::checked_case;
use serde_json::{Value, json};
type Check = Result<(), Box<dyn std::error::Error>>;
pub(super) fn check(
    capture: &str,
    args: &[&str],
    source: &str,
    pointer: &str,
    field: &str,
    retained: Value,
    missing: &str,
) -> Check {
    let positive = checked_case(capture, args, |_, _| {}, retained.clone(), 0)?;
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "native positive {source}: schema={}",
        positive
            .get("schema")
            .ok_or("native positive lacks schema")?
    )?;
    let mut edited = 0;
    let value = checked_case(
        capture,
        args,
        |name, record| {
            if name == source
                && selected_record(source, record)
                && let Some(value) = record.pointer_mut(pointer)
            {
                *value = if source == "orcid" {
                    json!(42)
                } else {
                    json!({"bad": true})
                };
                edited += 1;
            }
        },
        retained,
        0,
    )?;
    if edited != 1 {
        return Err(format!("expected one edited {source} field, got {edited}").into());
    }
    if value.pointer(missing).is_some() {
        return Err(format!("bad {source} field survived at {missing}").into());
    }
    let problems = value
        .get("problems")
        .and_then(Value::as_array)
        .ok_or("missing field problems")?;
    if problems
        .iter()
        .filter(|p| p["source"] == source && p["field"] == field)
        .count()
        != 1
    {
        return Err(format!("expected one {source} {field} problem: {value}").into());
    }
    Ok(())
}
// A capture directory may contain several original request identities. Change
// only the exact record selected by these two native positive controls.
fn selected_record(source: &str, record: &Value) -> bool {
    let selected = match source {
        "europepmc" => "/article/PPR/PPR1064174?",
        "semantic-scholar" => "/paper/PMID%3A33123537/references?",
        _ => return true,
    };
    record
        .get("url")
        .and_then(Value::as_str)
        .is_some_and(|url| url.contains(selected))
}
