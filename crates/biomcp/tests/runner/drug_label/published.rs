//! Feature drug-label: every source section remains selectable without changing its text.
use super::{Check, Value, both, field, get_args, get_setup, json};

pub(super) fn sections() -> Check {
    let original: Value = serde_json::from_str(&crate::runner::launch::fixture_text(
        "fixtures/drug/openfda/label-sections.json",
    )?)
    .map_err(|e| e.to_string())?;
    let record = original.pointer("/results/0").ok_or("source label")?;
    let (env, stub) = get_setup(record, record.clone())?;
    for heading in ["adverse_reactions", "adverse_reactions_table"] {
        let mut text = String::new();
        let mut offset = 0;
        loop {
            let value = both(&env, args(record, heading, offset)?, 0)?;
            text.push_str(
                value
                    .pointer("/result/label/data/text")
                    .and_then(Value::as_str)
                    .ok_or("section text")?,
            );
            let page = field(&value, "page");
            let returned = field(page, "returned").as_u64().ok_or("returned")?;
            let total = field(page, "total").as_u64().ok_or("total")?;
            if returned == 0 {
                return Err("section paging stalled".into());
            }
            offset += returned;
            if offset >= total {
                break;
            }
        }
        let expected = record
            .get(heading)
            .and_then(Value::as_array)
            .ok_or("native section")?
            .iter()
            .map(|v| v.as_str().ok_or("native text"))
            .collect::<Result<Vec<_>, _>>()?
            .concat();
        if text != expected {
            return Err(format!("source text changed: {heading}"));
        }
    }
    let absent = both(&env, args(record, "alarms", 0)?, 0)?;
    if absent.pointer("/result/label/state") != Some(&json!("empty"))
        || absent.pointer("/status") != Some(&json!("complete"))
        || absent.pointer("/sources/0/state") != Some(&json!("ok"))
    {
        return Err(
            "published but absent section must be empty with a complete source result".into(),
        );
    }
    for heading in ["not_a_section", "openfda", "id"] {
        let invalid = both(&env, args(record, heading, 0)?, 2)?;
        let actual = invalid
            .pointer("/error/valid_values")
            .ok_or("section choices")?;
        let expected: Vec<_> = record
            .as_object()
            .ok_or("record")?
            .iter()
            .filter(|(_, v)| v.is_array())
            .map(|(k, _)| json!(k))
            .collect();
        if actual != &json!(expected) {
            return Err(format!("wrong record choices: {actual}"));
        }
    }
    if stub.finish()?.len() != 1 {
        return Err("section selections refetched cached label".into());
    }
    Ok(())
}

fn args(record: &Value, heading: &str, offset: u64) -> Result<Vec<String>, String> {
    let mut args = get_args(record)?;
    args.extend([
        "label".into(),
        "--label-section".into(),
        heading.into(),
        "--section-offset".into(),
        offset.to_string(),
    ]);
    Ok(args)
}
