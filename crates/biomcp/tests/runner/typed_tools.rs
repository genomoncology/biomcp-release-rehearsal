tool_only! {
#[path = "typed_baseline.rs"]
mod baseline;
#[path = "typed_replay.rs"]
mod replay;
}
#[path = "typed_policy.rs"]
mod policy;
// Installed-artifact typed tool discovery, schema refusal and CLI equivalence.
use super::super::{
    http::Server,
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::io::Write;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("mcp-parity/typed-cli-bridge", || {
            super::typed_bridge::check().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-description-contract", || {
            description_contract().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-hosted-catalog", || {
            super::typed_hosted::check().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-help-research-discovery", || {
            super::typed_help::discovery().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-getter-help", || {
            getter_help().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-help-next-recovery", || {
            super::typed_help::recovery().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/compact-typed-schemas", || {
            let env = Environment::new()?;
            super::typed_schemas::check(&env).map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-reply-forms-and-cap", || {
            super::typed_reply::check().map_err(Into::into)
        }),
        transcript_trial(super::super::pool_contract::TYPED_TOOLS.0, || {
            check().map_err(Into::into)
        }),
    ]
}

tool_only! {
pub(super) fn proof_trials() -> Vec<Trial> {
    vec![
        transcript_trial("mcp-parity/typed-baseline-and-replay-guards", || {
            guards().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-historical-baseline-and-replay", || {
            historical().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/typed-clock-controls", || {
            clock_controls().map_err(Into::into)
        }),
    ]
}
}

pub(super) fn getter_help() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    policy::get_help(&env)
}

pub(super) fn description_contract() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed","mcp_typed_output_schema":true}))
        .map_err(|e| e.to_string())?;
    policy::check_descriptions(&env)
}

pub(super) fn check() -> Result<(), String> {
    checked_tools()?;
    policy::check()
}

fn checked_tools() -> Result<Vec<Value>, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    let globals = vec!["--offline".into()];
    let tools = discovery(&env, &globals)?;
    rejections(&env, &globals)?;
    Ok(tools)
}

tool_only! {
pub(super) fn historical() -> Result<(), String> {
    super::super::launch::typed_baseline()
        .ok_or("BIOMCP_TYPED_BASELINE is required for single-mode wire parity")?;
    let tools = checked_tools()?;

    let mut failures = Vec::new();
    let mut case_index = 0;
    for (verb, entity, _) in biomcp_types::command_paths() {
        let name = format!("{verb}_{entity}").replace('-', "_");
        let tool = super::typed_catalog::command(&tools, &name)
            .ok_or_else(|| format!("missing registry route {name}"))?;
        if let Err(error) = replay::check(&tool, &mut case_index) {
            failures.push(error);
        }
    }
    if let Err(error) = baseline::check() {
        failures.push(error);
    }
    if !failures.is_empty() {
        return Err(failures.join("\n"));
    }
    policy::check()?;
    Ok(())
}
}

fn discovery(env: &Environment, globals: &[String]) -> Result<Vec<Value>, String> {
    let replies = exchange_with_globals(
        env,
        &[
            request(json!(1), "tools/list", json!({})),
            request(json!(2), "server/discover", json!({})),
        ],
        globals,
    )?;
    let tools = replies
        .iter()
        .find(|r| r["id"] == 1)
        .and_then(|r| r.pointer("/result/tools"))
        .and_then(Value::as_array)
        .ok_or("missing tools")?;
    let paths = biomcp_types::command_paths();
    super::typed_catalog::check(tools)?;
    let instructions = replies
        .iter()
        .find(|r| r["id"] == 2)
        .and_then(|r| r.pointer("/result/instructions"))
        .and_then(Value::as_str)
        .ok_or("instructions")?;
    if instructions.chars().count() > 4096 {
        return Err("instruction budget exceeded".into());
    }
    for (verb, entity, _) in paths {
        let name = format!("{verb}_{entity}").replace('-', "_");
        let Some(tool) = tools.iter().find(|tool| tool["name"] == name) else {
            continue;
        };
        let description = tool["description"].as_str().ok_or("description")?;
        if description.chars().count() > 300 {
            return Err(format!(
                "{name} description {}: {description}",
                description.chars().count()
            ));
        }
    }
    writeln!(
        std::io::stdout(),
        "TYPED_TOOL_LIST characters={} tools={} response_characters={}",
        json!(tools).to_string().chars().count(),
        tools.len(),
        replies
            .iter()
            .find(|r| r["id"] == 1)
            .ok_or("tool-list reply")?
            .to_string()
            .chars()
            .count()
    )
    .map_err(|e| e.to_string())?;
    Ok(tools.clone())
}
fn rejections(env: &Environment, globals: &[String]) -> Result<(), String> {
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"mcp_tool_mode":"typed"})),
        None,
        None,
        None,
        globals,
    )?;
    for arguments in [
        json!({"source":"not-a-source"}),
        json!({"limit":"not-a-number"}),
        json!({"limit":true}),
        json!({"limit":null}),
        json!({"limit":{"value":10}}),
        json!({"unexpected":"value"}),
        json!([]),
    ] {
        let frame = request(
            json!(3),
            "tools/call",
            json!({"name":"search_trial","arguments":arguments}),
        );
        let stdio = validation_parity(env, globals, &server, &frame)?;
        let envelope = stdio
            .pointer("/result/structuredContent")
            .ok_or("usage envelope")?;
        if stdio.pointer("/result/isError") != Some(&json!(true))
            || envelope.get("status") != Some(&json!("usage"))
            || !envelope
                .pointer("/error/option")
                .is_some_and(Value::is_string)
            || !envelope
                .pointer("/error/valid_values")
                .is_some_and(Value::is_array)
        {
            return Err(format!("missing recoverable usage envelope: {stdio}"));
        }
        let text = stdio
            .pointer("/result/content/0/text")
            .and_then(Value::as_str)
            .ok_or("missing summary")?;
        let (_, next) = text
            .lines()
            .next()
            .ok_or("summary line")?
            .split_once("next ")
            .ok_or_else(|| format!("missing typed correction: {text}"))?;
        let next: Value = serde_json::from_str(next).map_err(|e| e.to_string())?;
        let frame = request(json!(4), "tools/call", next);
        let reply = validation_parity(env, globals, &server, &frame)?;
        if reply
            .pointer("/result/structuredContent/status")
            .and_then(Value::as_str)
            .is_none_or(|status| status == "usage")
        {
            return Err(format!("typed correction rejected: {reply}"));
        }
    }
    integer_spelling(env, globals, &server)
}

fn integer_spelling(env: &Environment, globals: &[String], server: &Server) -> Result<(), String> {
    let mut numeric_reply = None;
    for limit in [json!(10), json!("10")] {
        let frame = request(
            json!(5),
            "tools/call",
            json!({"name":"search_trial","arguments":{"limit":limit}}),
        );
        let reply = validation_parity(env, globals, server, &frame)?;
        let envelope = reply
            .pointer("/result/structuredContent")
            .ok_or("integer envelope")?;
        if envelope
            .get("status")
            .and_then(Value::as_str)
            .is_none_or(|status| status == "usage")
        {
            return Err(format!("CLI-compatible integer rejected: {reply}"));
        }
        if numeric_reply
            .as_ref()
            .is_some_and(|numeric| numeric != envelope)
        {
            return Err("string integer differs from numeric integer".into());
        }
        numeric_reply = Some(envelope.clone());
    }
    Ok(())
}

fn validation_parity(
    env: &Environment,
    globals: &[String],
    server: &Server,
    frame: &Value,
) -> Result<Value, String> {
    let replies = exchange_with_globals(env, std::slice::from_ref(frame), globals)?;
    let stdio = replies.first().ok_or("missing validation reply")?;
    let (status, http) = server.request("POST", "/mcp", &frame.to_string())?;
    let result = stdio
        .get("result")
        .ok_or_else(|| format!("missing tool result: {stdio}"))?;
    if status != 200 || Some(result) != http.get("result") {
        return Err(format!(
            "typed validation transports differ: {stdio} / {http}"
        ));
    }
    Ok(stdio.clone())
}

tool_only! {
pub(super) fn guards() -> Result<(), String> {
    super::super::launch::typed_baseline()
        .ok_or("BIOMCP_TYPED_BASELINE is required for single-mode wire parity")?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    let tools = discovery(&env, &["--offline".into()])?;
    let tool = super::typed_catalog::command(&tools, "search_biomarker")
        .ok_or("missing search_biomarker registry route")?;
    replay::rejects_overstrict_schema(&tool)?;
    let mut failures = Vec::new();
    if let Err(error) = baseline::invalid_id_witness() {
        failures.push(error);
    }
    if let Err(error) = baseline::check() {
        failures.push(error);
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

pub(super) fn clock_controls() -> Result<(), String> {
    super::super::launch::typed_baseline()
        .ok_or("BIOMCP_TYPED_BASELINE is required for single-mode wire parity")?;
    replay::clock_controls()
}

}
