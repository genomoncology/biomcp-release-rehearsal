use super::launch::Environment;
use serde_json::{Value, json};
use std::process::Stdio;

pub(super) fn metadata() -> Value {
    json!({"io.modelcontextprotocol/protocolVersion":"2026-07-28",
           "io.modelcontextprotocol/clientCapabilities":{}})
}

pub(super) fn request(id: Value, method: &str, mut params: Value) -> Value {
    if let Some(params) = params.as_object_mut() {
        params.insert("_meta".into(), metadata());
    }
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
}

pub(super) fn exchange(environment: &Environment, frames: &[Value]) -> Result<Vec<Value>, String> {
    exchange_with_globals(environment, frames, &[])
}
pub(super) fn exchange_with_globals(
    environment: &Environment,
    frames: &[Value],
    globals: &[String],
) -> Result<Vec<Value>, String> {
    exchange_retained(environment, frames, globals, None)
}
pub(super) fn exchange_retained(
    environment: &Environment,
    frames: &[Value],
    globals: &[String],
    directory: Option<&std::path::Path>,
) -> Result<Vec<Value>, String> {
    let mut args = vec!["serve".into()];
    args.extend_from_slice(globals);
    let mut transcript_args = args.clone();
    transcript_args.push("<requests>".into());
    transcript_args.extend(
        frames
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?,
    );
    let mut command =
        environment.command_with_arguments(env!("CARGO_BIN_EXE_biomcp"), &args, &transcript_args);
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let (status, stdout, stderr) = super::process::exchange(child, frames)?;
    command
        .record(status, &stdout, "mcp")
        .map_err(|e| e.to_string())?;
    if let Some(directory) = directory {
        std::fs::write(directory.join("mcp.stdout"), &stdout).map_err(|e| e.to_string())?;
        std::fs::write(directory.join("mcp.stderr"), &stderr).map_err(|e| e.to_string())?;
    }
    super::process::messages(status, &stdout, &stderr)
}

pub(super) fn call_with_globals(
    environment: &Environment,
    args: &[String],
    is_error: bool,
    globals: &[String],
) -> Result<Value, String> {
    call_retained(environment, args, is_error, globals, None)
}
pub(super) fn call_retained(
    environment: &Environment,
    args: &[String],
    is_error: bool,
    globals: &[String],
    directory: Option<&std::path::Path>,
) -> Result<Value, String> {
    call_text_retained(environment, args, is_error, globals, directory).map(|(value, _)| value)
}

pub(super) fn call_text_with_globals(
    environment: &Environment,
    args: &[String],
    is_error: bool,
    globals: &[String],
) -> Result<(Value, String), String> {
    call_text_retained(environment, args, is_error, globals, None)
}

fn call_text_retained(
    environment: &Environment,
    args: &[String],
    is_error: bool,
    globals: &[String],
    directory: Option<&std::path::Path>,
) -> Result<(Value, String), String> {
    let replies = exchange_retained(
        environment,
        &[request(
            json!(1),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args}}),
        )],
        globals,
        directory,
    )?;
    let reply = replies
        .first()
        .filter(|_| replies.len() == 1)
        .ok_or("MCP expected one reply")?;
    if reply.get("id") != Some(&json!(1)) {
        return Err("MCP correlation".into());
    }
    let result = reply
        .get("result")
        .ok_or_else(|| format!("MCP tool error: {reply}"))?;
    if result.get("isError") != Some(&json!(is_error)) {
        return Err("MCP is_error mismatch".into());
    }
    let structured = result
        .get("structuredContent")
        .ok_or("MCP missing structuredContent")?;
    let text = result
        .pointer("/content/0/text")
        .and_then(Value::as_str)
        .ok_or("MCP missing text")?;
    let parsed: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if &parsed != structured {
        return Err("MCP text differs from structuredContent".into());
    }
    Ok((structured.clone(), text.to_owned()))
}
