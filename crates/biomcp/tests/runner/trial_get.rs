//! Exact registry protocol integrity through real CLI/MCP and immutable page walks.
#![expect(
    clippy::indexing_slicing,
    reason = "Captured source fixtures and checked envelopes fail directly on shape drift"
)]
use super::{launch::Environment, response_cases::copy_capture};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
type Check = Result<(), Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<Trial> {
    // The test rejoins runtime Unicode/whitespace fragments and mutates bound snapshot inputs after recording access ends.
    vec![transcript_trial(
        crate::runner::pool_contract::TRIAL.0,
        || continuations().map_err(|e| e.to_string().into()),
    )]
}
pub(super) fn original(id: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let directory = super::launch::fixtures()
        .join("replies/trial/get")
        .join(id)
        .join("ctgov");
    for file in fs::read_dir(directory)? {
        let record: Value = serde_json::from_slice(&fs::read(file?.path())?)?;
        if record["url"].as_str().is_some_and(|url| url.ends_with(id)) {
            return Ok(record["exchanges"][0]["body"]["value"]["protocolSection"].clone());
        }
    }
    Err("detail recording missing".into())
}
pub(super) fn command(
    env: &Environment,
    args: &[String],
    globals: &[String],
    exit: i32,
) -> Result<Value, String> {
    let mut all = args.to_vec();
    all.extend_from_slice(globals);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &all)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) || !output.stderr.is_empty() {
        return Err(format!(
            "trial command exit {:?}, expected {exit}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    super::schema::Registry::embedded()?.validate(&value, args, exit == 0)?;
    Ok(value)
}

fn continuations() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    let original = format!(" \n{} Ω\n", "x  \n".repeat(3500));
    copy_capture("trial/get/NCT02296125", &replay, |_, record| {
        if let Some(timestamp) = record.pointer_mut("/exchanges/0/body/value/dataTimestamp") {
            *timestamp = json!("2024-02-30T10:00:00");
        }
        if record["url"]
            .as_str()
            .is_some_and(|u| u.ends_with("NCT02296125"))
        {
            let e = &mut record["exchanges"][0]["body"]["value"]["protocolSection"]["eligibilityModule"];
            e["eligibilityCriteria"] = json!(original);
            e.as_object_mut().map(|e| e.remove("maximumAge"));
        }
    })?;
    let args = ["get", "trial", "NCT02296125", "eligibility"].map(str::to_owned);
    let globals = env.globals(Some(&replay), false);
    let first = command(&env, &args, &globals, 0)?;
    processing_problem(&first)?;
    if first.pointer("/result/items/0/sections/eligibility/data/maximum_age/state")
        != Some(&json!("no_limit"))
    {
        return Err("missing maximum age invented a number".into());
    }
    let mut next = super::action(&first, "page")?;
    let mut joined = first
        .pointer("/result/items/0/sections/eligibility/data/criteria")
        .and_then(Value::as_str)
        .ok_or("missing text")?
        .to_owned();
    // No recording is available to continuations: only the admitted immutable snapshot can answer.
    let offline = env.globals(None, true);
    for _ in 0..10 {
        let out = command(&env, &next, &offline, 0)?;
        processing_problem(&out)?;
        let data = &out["result"]["items"][0]["sections"]["eligibility"]["data"];
        if data["offset"] != json!(joined.chars().count()) {
            return Err("whitespace fragment offset changed".into());
        }
        joined.push_str(data["criteria"].as_str().ok_or("fragment missing")?);
        match super::action(&out, "page") {
            Ok(args) => next = args,
            Err(_) => break,
        }
    }
    if joined != original {
        return Err("continuation changed source Unicode or whitespace".into());
    }
    let mut disabled = next.clone();
    disabled.extend(["--disable-sources".into(), "ctgov".into()]);
    let out = command(&env, &disabled, &offline, 4)?;
    if out["sources"][0]["reason"] != "disabled" || out.pointer("/result/items/0/card").is_some() {
        return Err("disabled source restored a stored protocol".into());
    }
    altered_pool(&env, &next, &offline)?;
    Ok(())
}
fn processing_problem(value: &Value) -> Check {
    let count = value
        .get("problems")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|problem| {
            problem.get("field") == Some(&json!("dataTimestamp"))
                && problem.get("code") == Some(&json!("value.unknown"))
        })
        .count();
    if count != 1 || value.pointer("/result/items/0/card/nct_id") != Some(&json!("NCT02296125")) {
        return Err(
            "processing timestamp problem or valid study did not survive paging exactly once"
                .into(),
        );
    }
    Ok(())
}
fn altered_pool(env: &Environment, args: &[String], globals: &[String]) -> Check {
    for selector in ["NCT02578680", "offset", "pool"] {
        let mut changed = args.to_vec();
        if selector.starts_with("NCT") {
            changed[2] = selector.into();
        } else {
            let value = changed
                .iter()
                .position(|s| s == &format!("--{selector}"))
                .ok_or("missing continuation selector")?
                + 1;
            changed[value] = if selector == "offset" {
                "4294967295".into()
            } else {
                "unknown-snapshot".into()
            };
        }
        let exit = if selector == "offset" { 2 } else { 3 };
        let out = command(env, &changed, globals, exit)?;
        if out["status"]
            != if selector == "offset" {
                "usage"
            } else {
                "expired"
            }
        {
            return Err("changed continuation was accepted".into());
        }
    }
    Ok(())
}
