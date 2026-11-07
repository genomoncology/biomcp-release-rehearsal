//! Usage corrections are executable requests on every public argument surface.
use super::{
    launch::Environment,
    mcp::{exchange_with_globals, request},
    transcript::trial,
};
use libtest_mimic::Trial;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Case {
    args: Vec<String>,
    tool: String,
    fields: Value,
    option: String,
    message: Option<String>,
    corrected_query: Option<Value>,
}

pub(super) fn trials() -> Vec<Trial> {
    vec![trial("query-validation/argument-recovery", || {
        check().map_err(Into::into)
    })]
}

fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let cases: Vec<Case> =
        serde_json::from_str(include_str!("argument_recovery.json")).map_err(|e| e.to_string())?;
    let mut expected = Vec::new();
    for case in &cases {
        let (code, value) = cli(&env, &case.args)?;
        if code != Some(2) {
            return Err(format!("usage exit differs: {value}"));
        }
        validate(&env, case, &value)?;
        expected.push(value);
    }
    for mode in ["single", "typed"] {
        check_mode(&env, &cases, &expected, mode)?;
    }
    let (_, retired) = cli(
        &env,
        &["search", "pgx", "--gene", "CYP2D6", "--source", "pharmgkb"].map(str::to_owned),
    )?;
    if retired.pointer("/error/code") != Some(&json!("usage.retired")) {
        return Err("obsolete source lost migration guidance".into());
    }
    Ok(())
}

fn check_mode(
    env: &Environment,
    cases: &[Case],
    expected: &[Value],
    mode: &str,
) -> Result<(), String> {
    let typed = mode == "typed";
    env.config(&json!({"mcp_tool_mode":mode,"mcp_typed_output_schema":true}))
        .map_err(|e| e.to_string())?;
    let tools = if typed { catalog(env)? } else { Vec::new() };
    let frames = cases
        .iter()
        .enumerate()
        .map(|(index, case)| {
            let params = if typed {
                let tool = super::protocol::typed_catalog::command(&tools, &case.tool)
                    .ok_or_else(|| format!("missing registered recovery route {}", case.tool))?;
                super::protocol::typed_catalog::call(&tool, &case.fields)
            } else {
                json!({"name":"biomcp","arguments":{"args":case.args}})
            };
            Ok(request(json!(index), "tools/call", params))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let replies = exchange_with_globals(env, &frames, &["--offline".into()])?;
    if replies.len() != frames.len() {
        return Err("missing usage replies".into());
    }
    for (index, (case, expected)) in cases.iter().zip(expected).enumerate() {
        let reply = replies
            .iter()
            .find(|reply| reply.get("id") == Some(&json!(index)))
            .ok_or("missing correlated usage reply")?;
        let value = reply
            .pointer("/result/structuredContent")
            .ok_or("missing usage envelope")?;
        let equal = if typed {
            super::protocol::typed_followups::data(value)
                == super::protocol::typed_followups::data(expected)
        } else {
            value == expected
        };
        if !equal || reply.pointer("/result/isError") != Some(&json!(true)) {
            return Err(format!("{mode} usage differs from CLI: {reply}"));
        }
        if typed {
            typed_correction(env, value, &tools)?;
        } else {
            validate(env, case, value)?;
        }
    }
    Ok(())
}
fn catalog(env: &Environment) -> Result<Vec<Value>, String> {
    let replies = exchange_with_globals(
        env,
        &[request(json!("catalog"), "tools/list", json!({}))],
        &["--offline".into()],
    )?;
    replies
        .first()
        .and_then(|reply| reply.pointer("/result/tools"))
        .and_then(Value::as_array)
        .cloned()
        .ok_or("missing typed catalog".into())
}

fn typed_correction(env: &Environment, value: &Value, tools: &[Value]) -> Result<(), String> {
    super::protocol::typed_followups::check_next(value, tools, &mut 0)?;
    let next = value
        .pointer("/next/0")
        .ok_or("missing typed corrected call")?;
    if next.get("name") == Some(&json!("help")) {
        return Err("expected a corrected request".into());
    }
    let replies = exchange_with_globals(
        env,
        &[request(
            json!(999),
            "tools/call",
            json!({"name":next["name"],"arguments":next["arguments"]}),
        )],
        &["--offline".into()],
    )?;
    let corrected = replies
        .first()
        .and_then(|reply| reply.pointer("/result/structuredContent"))
        .ok_or("missing typed corrected reply")?;
    if corrected.get("status") == Some(&json!("usage")) {
        return Err(format!("typed correction does not parse: {corrected}"));
    }
    Ok(())
}

fn cli(env: &Environment, args: &[String]) -> Result<(Option<i32>, Value), String> {
    let mut args = args.to_vec();
    args.push("--offline".into());
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.stderr.is_empty() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok((
        output.status.code(),
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?,
    ))
}

fn validate(env: &Environment, case: &Case, value: &Value) -> Result<(), String> {
    if case.message.as_ref().is_some_and(|message| {
        value.pointer("/error/message").and_then(Value::as_str) != Some(message)
    }) {
        return Err(format!("missing paging explanation: {value}"));
    }
    let option = value
        .pointer("/error/option")
        .and_then(Value::as_str)
        .map(|name| name.strip_prefix("--").unwrap_or(name));
    let expected = case.option.strip_prefix("--").unwrap_or(&case.option);
    if value.pointer("/error/code") != Some(&json!("usage.arguments"))
        || option != Some(expected)
        || value
            .pointer("/error/valid_values")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
    {
        return Err(format!("missing argument guidance: {value}"));
    }
    let args: Vec<String> = value
        .pointer("/next/0/args")
        .and_then(Value::as_array)
        .ok_or("missing corrected command")?
        .iter()
        .map(|word| {
            word.as_str()
                .map(str::to_owned)
                .ok_or("command word".to_owned())
        })
        .collect::<Result<_, _>>()?;
    if args.first().is_none_or(|word| word == "help") {
        return Err("expected a corrected request".into());
    }
    let (_, corrected) = cli(env, &args)?;
    if let Some(fields) = case.corrected_query.as_ref().and_then(Value::as_object) {
        for (name, expected) in fields {
            if corrected.get("query").and_then(|query| query.get(name)) != Some(expected) {
                return Err(format!("correction changed selector {name}: {corrected}"));
            }
        }
    }
    if corrected.get("status") == Some(&json!("usage")) {
        return Err(format!("correction does not parse: {corrected}"));
    }
    Ok(())
}
