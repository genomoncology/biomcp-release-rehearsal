//! Separate calls may retrieve the same report page at different Unix seconds.
use biomcp_types::{AdverseEventPage, ResponseBody};
use serde_json::{Value, json};

pub(super) fn same_reply(left: &Value, right: &Value) -> Result<bool, String> {
    Ok(comparable(left)? == comparable(right)?)
}

fn comparable(value: &Value) -> Result<Value, String> {
    let mut value = value.clone();
    if value.get("schema").and_then(Value::as_str) == Some(AdverseEventPage::SCHEMA)
        && let Some(fetched) = value.pointer_mut("/result/fetched")
    {
        if fetched.as_u64().is_none() {
            return Err(
                "report page fetched must be retrieval time in unsigned Unix seconds".into(),
            );
        }
        // Keep presence distinct from legacy recordings with no acquisition time.
        *fetched = json!(0);
    }
    Ok(value)
}

tool_only! {
pub(super) fn trial() -> libtest_mimic::Trial {
    super::transcript::trial("runner/report-parity-keeps-every-nonvolatile-field", || {
        proof().map_err(Into::into)
    })
}

fn proof() -> Result<(), String> {
    let left = json!({"schema":AdverseEventPage::SCHEMA,"result":{
        "fetched":1793298499_u64,"source_updated":"2024-09-01","items":[],"retrieved_count":2
    }});
    let mut right = left.clone();
    *right
        .pointer_mut("/result/fetched")
        .ok_or("fetched field")? = json!(1793298505_u64);
    if !same_reply(&left, &right)? {
        return Err("separate retrieval seconds changed report parity".into());
    }
    for malformed in [json!(-1), json!(1.5), json!("1793298505"), Value::Null] {
        *right
            .pointer_mut("/result/fetched")
            .ok_or("fetched field")? = malformed;
        if same_reply(&left, &right).is_ok() || same_reply(&right, &left).is_ok() {
            return Err("malformed retrieval timestamp was ignored".into());
        }
    }
    for (field, changed) in [
        ("source_updated", json!("2024-09-02")),
        ("retrieved_count", json!(3)),
        ("items", json!([{"id":"changed"}])),
        ("updated", json!("2024-09-02")),
    ] {
        right = left.clone();
        right
            .get_mut("result")
            .and_then(Value::as_object_mut)
            .ok_or("result object")?
            .insert(field.into(), changed);
        if same_reply(&left, &right)? {
            return Err(format!("report parity ignored {field}"));
        }
    }
    right = left.clone();
    right
        .get_mut("result")
        .and_then(Value::as_object_mut)
        .ok_or("result object")?
        .remove("fetched");
    if same_reply(&left, &right)? {
        return Err("missing retrieval time compared equal to a known time".into());
    }
    let mut other = left;
    *other.get_mut("schema").ok_or("schema field")? = json!("unrelated/1");
    right = other.clone();
    *right
        .pointer_mut("/result/fetched")
        .ok_or("fetched field")? = json!(1793298505_u64);
    if same_reply(&other, &right)? {
        return Err("report parity ignored another schema's field".into());
    }
    Ok(())
}
}
