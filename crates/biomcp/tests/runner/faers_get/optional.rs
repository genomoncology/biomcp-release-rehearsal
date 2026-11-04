//! Optional FAERS values retain identity and independent native sections.
use super::Check;
use serde_json::{Value, json};

pub(super) fn input(body: &mut Value, mode: &str) -> Check {
    if mode == "empty-collections" {
        for field in ["drug", "reaction"] {
            *body
                .pointer_mut(&format!("/results/0/patient/{field}"))
                .ok_or(field)? = json!([]);
        }
        return Ok(());
    }
    let Some((kind, field)) = mode.split_once('-') else {
        return Ok(());
    };
    if !matches!(kind, "missing" | "null" | "invalid") {
        return Ok(());
    }
    let (path, key) = match field {
        "patient" => ("/results/0", "patient"),
        "drugs" => ("/results/0/patient", "drug"),
        "reactions" => ("/results/0/patient", "reaction"),
        _ => return Err(format!("unknown optional field: {mode}")),
    };
    let object = body
        .pointer_mut(path)
        .and_then(Value::as_object_mut)
        .ok_or(path)?;
    if kind == "missing" {
        object.remove(key);
    } else {
        object.insert(
            key.into(),
            if kind == "null" {
                Value::Null
            } else {
                json!(false)
            },
        );
    }
    Ok(())
}
