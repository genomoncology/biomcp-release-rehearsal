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
        env.config(&json!({"mcp_tool_mode":mode}))
            .map_err(|e| e.to_string())?;
        let frames: Vec<_> = cases
            .iter()
            .enumerate()
            .map(|(index, case)| {
                let params = if mode == "typed" {
                    json!({"name":case.tool,"arguments":case.fields})
                } else {
                    json!({"name":"biomcp","arguments":{"args":case.args}})
                };
                request(json!(index), "tools/call", params)
            })
            .collect();
        let replies = exchange_with_globals(&env, &frames, &["--offline".into()])?;
        if replies.len() != cases.len() {
            return Err("missing usage replies".into());
        }
        for (index, (case, expected)) in cases.iter().zip(&expected).enumerate() {
            let reply = replies
                .iter()
                .find(|reply| reply.get("id") == Some(&json!(index)))
                .ok_or("missing correlated usage reply")?;
            let value = reply
                .pointer("/result/structuredContent")
                .ok_or("missing usage envelope")?;
            if value != expected || reply.pointer("/result/isError") != Some(&json!(true)) {
                return Err(format!("{mode} usage differs from CLI: {reply}"));
            }
            validate(&env, case, value)?;
        }
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
    if corrected.get("status") == Some(&json!("usage")) {
        return Err(format!("correction does not parse: {corrected}"));
    }
    Ok(())
}
