//! Feature drug-approvals: newest submissions and intact source-backed pages.
#![expect(
    clippy::indexing_slicing,
    reason = "Captured source rows fail directly on shape drift"
)]
use super::{launch::Environment, response_cases::copy_capture, trial_get::command};
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        super::transcript_trial("drug-approvals/newest-submission-pages", || {
            newest().map_err(|e| e.to_string().into())
        }),
        super::transcript_trial("drug-approvals/undated-and-numeric-ties", || {
            controls().map_err(|e| e.to_string().into())
        }),
    ]
}
fn args() -> Vec<String> {
    [
        "get",
        "drug",
        "NDA021588",
        "approvals",
        "--source",
        "drugsfda",
    ]
    .map(str::to_owned)
    .to_vec()
}
fn newest() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("drug/approvals-newest", &replay, |_, _| {})?;
    let first = command(&env, &args(), &env.globals(Some(&replay), false), 0)?;
    if first["page"]["returned"] != 10
        || first["page"]["total"] != 47
        || first["result"]["approvals"]["data"]["records"][0]["submission_number"] != "67"
        || first["result"]["approvals"]["data"]["records"][0]["event"]["dates"][0]["date"]
            != "2026-07-13"
        || first["result"]["order"]
            != "submission status date descending, then submission number descending; missing dates last"
    {
        return Err(
            "default approvals page must expose the newest ten source submissions and its order"
                .into(),
        );
    }
    let rows = walk(&env, first)?;
    let expected: Value = serde_json::from_slice(&std::fs::read(
        super::launch::fixtures().join("replies/drug/approvals-newest/expected-order.json"),
    )?)?;
    if json!(rows) != expected {
        return Err(
            "submission pages repeated, skipped or reordered a source status date/number".into(),
        );
    }
    Ok(())
}
fn walk(env: &Environment, mut current: Value) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    loop {
        let page = current["result"]["approvals"]["data"]["records"]
            .as_array()
            .ok_or("submission page missing")?;
        if current["page"]["offset"].as_u64() != Some(rows.len() as u64)
            || page.is_empty()
            || page.len() > 10
        {
            return Err("submission offset, default page size or progress changed".into());
        }
        rows.extend(page.iter().map(|r| json!({
            "submission_type":r["event"]["kind"],
            "submission_number":r["submission_number"],
            "submission_status_date":r.pointer("/event/dates/0/date").and_then(Value::as_str).map(|s|s.replace('-', "")),
        })));
        if rows.len() == 47 {
            break;
        }
        let next = super::action(&current, "page")?;
        current = command(env, &next, &env.globals(None, true), 0)?;
    }
    Ok(rows)
}
fn controls() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("drug/approvals-newest", &replay, |_, record| {
        if let Some(rows) = record
            .pointer_mut("/exchanges/0/body/value/results/0/submissions")
            .and_then(Value::as_array_mut)
        {
            rows.truncate(3);
            for (row, number) in rows.iter_mut().zip(["2", "10", "100"]) {
                row["submission_number"] = json!(number);
                row["submission_status_date"] = json!("20260713");
            }
            rows[2]
                .as_object_mut()
                .map(|row| row.remove("submission_status_date"));
        }
    })?;
    let reply = command(&env, &args(), &env.globals(Some(&replay), false), 0)?;
    let rows = reply["result"]["approvals"]["data"]["records"]
        .as_array()
        .ok_or("submissions missing")?;
    let numbers: Vec<_> = rows
        .iter()
        .map(|r| r["submission_number"].clone())
        .collect();
    if numbers != vec![json!("10"), json!("2"), json!("100")]
        || rows[2].pointer("/event/dates/0").is_some()
        || reply["page"]["total"] != 3
    {
        return Err(
            "same-date submission numbers need numerical order and missing dates must remain last"
                .into(),
        );
    }
    Ok(())
}
