//! Typed follow-ups remain executable calls across replay, errors and reply forms.
//! live_check: local transport replay using public recorded gene replies; no live calls.
use super::super::{
    http::Server,
    launch::{Environment, fixtures},
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};
#[path = "typed_owned_followups.rs"]
mod owned;
#[path = "../../src/schema/steps.rs"]
mod step_paths;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    let mut trials = vec![crate::runner::transcript::trial(
        "mcp-parity/typed-followups",
        || check().map_err(Into::into),
    )];
    trials.extend(owned::trials());
    trials
}

pub(super) fn check() -> Result<(), String> {
    for form in ["structured-only", "text-only", "both"] {
        for hosted in [false, true] {
            check_form(form, hosted, false)?;
        }
    }
    check_form("both", false, true)?;
    check_form("both", true, true)?;
    reference_admission()?;
    Ok(())
}

fn reference_admission() -> Result<(), String> {
    let config = json!({"mcp_tool_mode":"typed","mcp_typed_enum_limit":1,
        "disable_sources":["civic","gmdn"]});
    let env = Environment::new().map_err(|error| error.to_string())?;
    env.config(&config).map_err(|error| error.to_string())?;
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(config),
        None,
        None,
        None,
        &["--offline".into()],
    )?;
    for (index, arguments) in [
        json!({"command":"biomcp","arguments":{"args":["help"]}}),
        json!({"command":"biomcp_help","arguments":{"command":"get gene"}}),
        json!({"command":"get_gene","arguments":{"input":["BRAF"]}}),
        json!({"command":"unknown","arguments":{}}),
        json!({"command":"get_pathway","arguments":{"id":"hsa04151"},"extra":true}),
    ]
    .into_iter()
    .enumerate()
    {
        let frame = request(
            json!(index),
            "tools/call",
            json!({"name":"biomcp_reference","arguments":arguments}),
        );
        let stdio =
            exchange_with_globals(&env, std::slice::from_ref(&frame), &["--offline".into()])?;
        let (status, http) = server.request("POST", "/mcp", &frame.to_string())?;
        if status != 200
            || stdio.first().and_then(|reply| reply.pointer("/error/code")) != Some(&json!(-32602))
            || http.pointer("/error/code") != Some(&json!(-32602))
        {
            return Err(format!(
                "reference accepted an unadvertised or malformed route: {stdio:?} {http}"
            ));
        }
    }
    Ok(())
}
fn check_form(form: &str, hosted: bool, restricted: bool) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let disabled = if restricted {
        vec!["oncokb", "civic"]
    } else {
        vec!["oncokb"]
    };
    let config = json!({"mcp_tool_mode":"typed", "mcp_typed_output_schema":true,"mcp_typed_definition_bytes":200000,
        "mcp_reply_form":form,"disable_sources":disabled,
        "mcp_typed_enum_limit":if restricted {1} else {30}});
    env.config(&config).map_err(|e| e.to_string())?;
    let replay = fixtures().join("replies/gene/main");
    let mut globals = env.globals(Some(&replay), false);
    let hosted_globals = if hosted {
        vec!["--hosted".into()]
    } else {
        vec![]
    };
    globals.extend(hosted_globals.clone());
    globals.extend([
        "--config".into(),
        env.root()
            .join("config/biomcp/config.json")
            .to_string_lossy()
            .into_owned(),
    ]);
    let frames = [
        request(json!(1), "tools/list", json!({})),
        request(
            json!(2),
            "tools/call",
            json!({"name":"get_gene","arguments":{"input":["BRAF"]}}),
        ),
        request(
            json!(3),
            "tools/call",
            json!({"name":"get_gene","arguments":{}}),
        ),
    ];
    let replies = exchange_with_globals(&env, &frames, &globals)?;
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(config),
        Some(replay),
        None,
        None,
        &hosted_globals,
    )?;
    let tools = replies
        .iter()
        .find(|v| v["id"] == 1)
        .and_then(|v| v.pointer("/result/tools"))
        .and_then(Value::as_array)
        .ok_or("tools")?;
    for (index, frame) in frames.iter().enumerate().skip(1) {
        let (status, http) = server.request("POST", "/mcp", &frame.to_string())?;
        if status != 200 {
            return Err(format!("HTTP {status}: {http}"));
        }
        let stdio = replies
            .iter()
            .find(|v| v["id"] == frame["id"])
            .ok_or("stdio reply")?;
        for reply in [stdio, &http] {
            check_reply(reply, tools, index == 1 && !restricted, restricted)?;
        }
    }
    Ok(())
}

fn check_reply(
    reply: &Value,
    tools: &[Value],
    section_expected: bool,
    restricted: bool,
) -> Result<(), String> {
    let text = reply
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .ok_or("text")?;
    let value = reply
        .pointer("/result/structuredContent")
        .cloned()
        .or_else(|| serde_json::from_str(text.split_once('\n')?.1).ok())
        .ok_or("typed body")?;
    let mut sections = 0;
    check_next(&value, tools, &mut sections)?;
    if restricted
        && value
            .get("next")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|call| call.pointer("/arguments/section"))
            .any(|section| {
                section == "civic"
                    || section
                        .as_array()
                        .is_some_and(|sections| sections.contains(&json!("civic")))
            })
    {
        return Err("lowering the public enum limit admitted a disabled CIViC follow-up".into());
    }
    if section_expected && sections == 0 {
        return Err("gene replay supplied no typed section follow-up".into());
    }
    if text.contains("--config") || text.contains("--replay") || text.contains("--case-cache") {
        return Err(format!("server arguments leaked: {text}"));
    }
    let owner = tools
        .iter()
        .find(|tool| tool["name"] == "get_gene")
        .ok_or("get_gene")?;
    jsonschema::validator_for(&owner["outputSchema"])
        .map_err(|e| e.to_string())?
        .validate(&value)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(in crate::runner) fn check_next(
    value: &Value,
    tools: &[Value],
    sections: &mut usize,
) -> Result<(), String> {
    let Some(schema) = response_schema(value) else {
        return Ok(());
    };
    for path in step_paths::paths(&schema, &schema, value) {
        let call = value.pointer(&path).ok_or("owned follow-up")?;
        if call.get("args").is_some() {
            return Err(format!("CLI follow-up survived at {path}: {call}"));
        }
        let tool = tools
            .iter()
            .find(|tool| tool.get("name") == call.get("name"))
            .ok_or_else(|| format!("unadvertised follow-up: {call}"))?;
        jsonschema::validator_for(&tool["inputSchema"])
            .map_err(|e| e.to_string())?
            .validate(&call["arguments"])
            .map_err(|e| format!("{call}: {e}"))?;
        if call.get("kind") == Some(&json!("section")) {
            if call
                .pointer("/arguments/section")
                .or_else(|| call.pointer("/arguments/arguments/section"))
                .is_none()
            {
                return Err(format!("section stayed in input: {call}"));
            }
            *sections += 1;
        }
    }
    Ok(())
}
fn response_schema(value: &Value) -> Option<Value> {
    let id = value.get("schema")?.as_str()?;
    let (_, text) = crate::runner::schema::ARTIFACTS
        .iter()
        .find(|(name, _)| *name == id)?;
    serde_json::from_str(text).ok()
}

// Other typed parity tests compare shared data while this test owns the next-call contract.
pub(in crate::runner) fn data(value: &Value) -> Value {
    let mut value = value.clone();
    if let Some(schema) = response_schema(&value) {
        for path in step_paths::paths(&schema, &schema, &value)
            .into_iter()
            .rev()
        {
            step_paths::remove(&mut value, &path);
        }
    }
    value
}
