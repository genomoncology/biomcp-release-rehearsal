#[path = "hosted_surface.rs"]
mod hosted_surface;
#[path = "source_age.rs"]
mod source_age;

#[path = "label_priority.rs"]
mod label_priority;
#[path = "mcp_reply_form.rs"]
mod reply_form;
#[path = "protocol_schemas.rs"]
mod schemas;
#[path = "typed_tools.rs"]
mod typed;
#[path = "typed_bridge.rs"]
mod typed_bridge;
#[path = "typed_catalog.rs"]
pub(super) mod typed_catalog;
#[path = "typed_entity.rs"]
mod typed_entity;
#[path = "typed_followups.rs"]
pub(super) mod typed_followups;
#[path = "typed_help.rs"]
mod typed_help;
#[path = "typed_hosted.rs"]
mod typed_hosted;
#[path = "typed_reply.rs"]
mod typed_reply;
#[path = "typed_schemas.rs"]
mod typed_schemas;
use super::{
    launch::Environment,
    mcp::{exchange, request},
    process::Process,
};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{io::Write, process::Stdio, time::Instant};

type Check = Result<(), String>;

pub(super) fn trials() -> Vec<Trial> {
    let mut trials = typed_followups::trials();
    trials.extend(hosted_surface::trials());
    trials.extend(typed::trials());
    trials.extend(source_age::trials());
    trials.extend([
        transcript_trial("mcp-parity/cancellation-suppresses-later-output", || {
            cancellation().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/versions-and-validation", || {
            validation().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/correlated-calls-drain-on-eof", || {
            concurrent().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/offline-observations-reuse", || {
            offline_observations().map_err(Into::into)
        }),
    ]);
    trials.extend(super::held_proxy::trials());
    trials.extend([
        transcript_trial("mcp-parity/bounded-framing", || {
            framing().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/broken-output-with-open-input", || {
            broken_output().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/discovers-artifact-command-tree", || {
            discovery().map_err(Into::into)
        }),
    ]);
    trials.extend(label_priority::trials());
    tool_only! {
        trials.extend(typed::proof_trials());
    }
    trials.extend(reply_form::trials());
    trials.extend(super::store_clean::trials());
    trials.extend(super::store_coexistence::trials());
    trials
}

fn environment() -> Result<Environment, String> {
    Environment::new().map_err(|e| e.to_string())
}

fn reply<'a>(replies: &'a [Value], id: &Value) -> Result<&'a Value, String> {
    let mut found = replies.iter().filter(|r| r.get("id") == Some(id));
    let result = found
        .next()
        .ok_or_else(|| format!("missing reply {id}: {replies:?}"))?;
    if found.next().is_some() {
        return Err(format!("duplicate reply {id}"));
    }
    Ok(result)
}

fn validation_frames() -> Result<Vec<Value>, String> {
    let mut unsupported = request(json!(2), "ping", json!({}));
    unsupported
        .pointer_mut("/params/_meta/io.modelcontextprotocol~1protocolVersion")
        .ok_or("metadata missing")?
        .clone_from(&json!("1900-01-01"));
    let frames = vec![
        request(json!(1), "server/discover", json!({})),
        unsupported,
        json!({"jsonrpc":"2.0","id":3,"method":"ping"}),
        request(json!(4), "unknown", json!({})),
        request(
            json!(5),
            "tools/call",
            json!({"name":"other","arguments":{"args":[]}}),
        ),
        request(
            json!(6),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":[1]}}),
        ),
        request(
            json!(7),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":[],"extra":true}}),
        ),
        json!({"jsonrpc":"2.0","id":8,"method":"initialize","params":{
            "protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"unknown","reason":"SENSITIVE-cancel"}}),
        json!({"jsonrpc":"2.0","method":"unknown"}),
        json!({"jsonrpc":"2.0","id":9,"method":"ping"}),
        request(json!(10), "ping", json!({})),
        json!({"jsonrpc":"2.0","id":11,"method":"tools/call","params":{"name":"biomcp","arguments":{"args":["schema","missing/1"]}}}),
        json!({"jsonrpc":"2.0","id":null,"method":"ping"}),
        json!({"jsonrpc":"2.0","id":1.5,"method":"ping"}),
        json!([]),
    ];
    // Required metadata is checked independently even after legacy initialize.
    let mut frames = frames;
    frames.push(
        json!({"jsonrpc":"2.0","id":12,"method":"ping","params":{"_meta":{
        "io.modelcontextprotocol/clientCapabilities":{}}}}),
    );
    frames.push(request(
        json!(13),
        "tools/call",
        json!({"name":"biomcp","arguments":{}}),
    ));
    frames.push(request(
        json!(14),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":"schema"}}),
    ));
    frames.push(request(
        json!(15),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":null}}),
    ));
    frames.push(json!({"jsonrpc":"2.0","id":16,"method":"ping","params":[]}));
    frames.push(json!({"jsonrpc":"2.0", "id":"invalid-method", "method":9}));
    for (index, malformed) in [Value::Null, json!([]), json!("SENSITIVE-meta")]
        .into_iter()
        .enumerate()
    {
        frames.push(json!({"jsonrpc":"2.0", "id":20 + index, "method":"ping",
            "params":{"_meta":malformed}}));
    }
    for (index, malformed) in [Value::Null, json!([]), json!(7)].into_iter().enumerate() {
        frames.push(request(
            json!(23 + index),
            "tools/list",
            json!({"cursor":malformed}),
        ));
    }
    Ok(frames)
}

fn validation() -> Check {
    let frames = validation_frames()?;
    let replies = exchange(&environment()?, &frames)?;
    if replies.len() != frames.len() - 3 {
        return Err(format!("notification response: {replies:?}"));
    }
    for (id, code) in [
        (2, -32022),
        (3, -32602),
        (4, -32601),
        (5, -32602),
        (6, -32602),
        (7, -32602),
        (12, -32602),
        (13, -32602),
        (14, -32602),
        (15, -32602),
        (16, -32602),
        (20, -32602),
        (21, -32602),
        (22, -32602),
        (23, -32602),
        (24, -32602),
        (25, -32602),
    ] {
        if reply(&replies, &json!(id))?.pointer("/error/code") != Some(&json!(code)) {
            return Err(format!("wrong validation code for {id}: {replies:?}"));
        }
    }
    if reply(&replies, &json!(8))?.pointer("/result/protocolVersion") != Some(&json!("2025-11-25"))
        || reply(&replies, &json!(10))?.pointer("/result/resultType") != Some(&json!("complete"))
        || reply(&replies, &json!(11))?.pointer("/result/isError") != Some(&json!(true))
    {
        return Err(format!("era adapters: {replies:?}"));
    }
    if reply(&replies, &json!("invalid-method"))?.pointer("/error/code") != Some(&json!(-32600)) {
        return Err("invalid request must preserve its valid correlation ID".into());
    }
    let versions = json!(["2026-07-28", "2025-11-25", "2025-06-18", "2025-03-26"]);
    if reply(&replies, &json!(1))?.pointer("/result/supportedVersions") != Some(&versions)
        || reply(&replies, &json!(2))?.pointer("/error/data/supported") != Some(&versions)
    {
        return Err("supported version lists differ".into());
    }
    if replies
        .iter()
        .filter(|r| r.pointer("/error/code") == Some(&json!(-32600)))
        .count()
        != 4
    {
        return Err("invalid IDs and batch must be refused".into());
    }
    if serde_json::to_string(&replies)
        .map_err(|e| e.to_string())?
        .contains("SENSITIVE")
    {
        return Err("notification leaked".into());
    }
    Ok(())
}

fn concurrent() -> Check {
    let ids = [json!("one"), json!(2), json!("three"), json!(4)];
    let mut frames: Vec<_> = ids
        .iter()
        .map(|id| {
            request(
                id.clone(),
                "tools/call",
                json!({"name":"biomcp","arguments":{"args":["schema","biomcp.problem/2"]}}),
            )
        })
        .collect();
    frames.push(request(
        json!("gwas"),
        "tools/call",
        json!({"name":"biomcp",
        "arguments":{"args":["search","gwas","--pubmed","17529967"]}}),
    ));
    let env = environment()?;
    let replay = super::launch::fixtures().join("replies/gwas/synthetic");
    let replies =
        super::mcp::exchange_with_globals(&env, &frames, &env.globals(Some(&replay), false))?;
    if replies.len() != ids.len() + 1 {
        return Err("EOF failed to drain accepted calls".into());
    }
    if reply(&replies, &json!("gwas"))?.pointer("/result/structuredContent/result/items/0/id")
        != Some(&json!("association:6110"))
    {
        return Err("concurrent GWAS cache-writing call failed".into());
    }
    if !env.root().join("case-cache/replay.sqlite").is_file() {
        return Err("GWAS call did not write its owned cache".into());
    }
    for id in ids {
        let result = reply(&replies, &id)?
            .get("result")
            .ok_or("missing result")?;
        if result.pointer("/structuredContent/$id") != Some(&json!("biomcp.problem/2")) {
            return Err(format!("incomplete response {result}"));
        }
        let text = result
            .pointer("/content/0/text")
            .and_then(Value::as_str)
            .ok_or("text missing")?;
        if serde_json::from_str::<Value>(text).map_err(|e| e.to_string())?
            != result
                .get("structuredContent")
                .cloned()
                .ok_or("structured missing")?
        {
            return Err("concurrent content differs".into());
        }
    }
    Ok(())
}

fn raw(bytes: &[u8]) -> Result<(std::process::ExitStatus, Vec<u8>, Vec<u8>), String> {
    let env = environment()?;
    let child = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &["serve".into()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut process = Process::new(child)?;
    let mut stdin = process.child.stdin.take().ok_or("stdin missing")?;
    let _ = stdin.write_all(bytes);
    drop(stdin);
    process.finish()
}

fn framing() -> Check {
    for (bytes, code) in [
        (b"{SENSITIVE-malformed\n".to_vec(), -32700),
        (
            b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}".to_vec(),
            -32700,
        ),
        (
            format!("SENSITIVE-oversize{}\\n", "x".repeat(128 * 1024)).into_bytes(),
            -32600,
        ),
    ] {
        let (status, stdout, stderr) = raw(&bytes)?;
        let response: Value = serde_json::from_slice(&stdout).map_err(|e| e.to_string())?;
        if !status.success()
            || !stderr.is_empty()
            || response.pointer("/error/code") != Some(&json!(code))
            || String::from_utf8_lossy(&stdout).contains("SENSITIVE")
        {
            return Err(format!(
                "fixed framing error expected: {response}, {status}"
            ));
        }
    }
    Ok(())
}

fn broken_output() -> Check {
    let started = Instant::now();
    let (reader, writer) = std::io::pipe().map_err(|e| e.to_string())?;
    drop(reader);
    let env = environment()?;
    let child = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &["serve".into()])
        .stdin(Stdio::piped())
        .stdout(writer)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut child = child;
    let mut stdin = child.stdin.take().ok_or("stdin missing")?;
    let mut bytes =
        serde_json::to_vec(&request(json!(1), "ping", json!({}))).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    stdin.write_all(&bytes).map_err(|e| e.to_string())?;
    let (status, _, stderr) = Process::new(child)?.finish()?;
    drop(stdin);
    if status.code() != Some(5) || stderr != b"biomcp: output failed\n" {
        return Err(format!(
            "broken stdout must exit 5 with fixed diagnostic while stdin stays open; \
             elapsed={:?}; status={status}; stderr (first {} of {} bytes)={:?}",
            started.elapsed(),
            stderr.len().min(4096),
            stderr.len(),
            String::from_utf8_lossy(stderr.get(..4096).unwrap_or(&stderr)),
        ));
    }
    Ok(())
}

fn discovery() -> Check {
    let replies = exchange(
        &environment()?,
        &[request(json!(1), "tools/list", json!({}))],
    )?;
    let tools = reply(&replies, &json!(1))?
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .ok_or("tools missing")?;
    let tool = tools
        .first()
        .filter(|_| tools.len() == 3)
        .ok_or("expected three tools")?;
    if tool.get("name") != Some(&json!("biomcp"))
        || tool.get("inputSchema")
            != Some(&json!({
                "$schema":"https://json-schema.org/draft/2020-12/schema",
                "type":"object",
                "properties":{"args":{"type":"array","items":{"type":"string"},
                    "description":"CLI command words, without biomcp. Example: [\"help\"]"}},
                "required":["args"],
                "additionalProperties":false
            }))
    {
        return Err("tool contract differs".into());
    }
    schemas::check(tools, reply(&replies, &json!(1))?)?;
    let text = tool
        .get("description")
        .and_then(Value::as_str)
        .ok_or("description missing")?;
    let mut descriptions = 0;
    for (name, key) in [("search", "query"), ("fetch", "id")] {
        let wrapper = tools
            .iter()
            .find(|tool| tool.get("name") == Some(&json!(name)))
            .ok_or("research wrapper missing")?;
        descriptions += wrapper
            .get("description")
            .and_then(Value::as_str)
            .ok_or("wrapper description missing")?
            .len();
        if wrapper.pointer("/inputSchema/required") != Some(&json!([key]))
            || wrapper.pointer("/annotations/readOnlyHint") != Some(&json!(true))
        {
            return Err("research tool contract differs".into());
        }
    }
    writeln!(
        std::io::stdout(),
        "RESEARCH_DESCRIPTION_BYTES={descriptions}"
    )
    .map_err(|e| e.to_string())?;
    if descriptions > crate::runner::output_limit("research-descriptions")? {
        return Err("research descriptions exceed budget".into());
    }
    discovery_description(text)
}

fn discovery_description(text: &str) -> Check {
    writeln!(std::io::stdout(), "MCP_DESCRIPTION_BYTES={}", text.len())
        .map_err(|e| e.to_string())?;
    if text.len() > crate::runner::output_limit("command-description")? {
        return Err("command description exceeds budget".into());
    }
    let example = text
        .split("Example: ")
        .nth(1)
        .and_then(|rest| rest.split(". Start").next())
        .ok_or("description has no copyable example")?;
    let arguments: Value = serde_json::from_str(example).map_err(|e| e.to_string())?;
    let replies = exchange(
        &environment()?,
        &[request(
            json!("example"),
            "tools/call",
            json!({"name":"biomcp","arguments":arguments}),
        )],
    )?;
    if reply(&replies, &json!("example"))?.pointer("/result/isError") != Some(&json!(false)) {
        return Err("call written from tool description failed".into());
    }
    Ok(())
}

// A call can finish before cancellation arrives. A following correlated ping
// proves the notification was consumed; no tool reply may follow that marker.
fn cancellation() -> Check {
    let frames = [
        request(
            json!("cancel-me"),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":["schema","biomcp.problem/2"]}}),
        ),
        json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"cancel-me","reason":"SENSITIVE-cancel-reason"}}),
        request(json!("marker"), "ping", json!({})),
        json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"already-completed"}}),
    ];
    let replies = exchange(&environment()?, &frames)?;
    reply(&replies, &json!("marker"))?;
    let marker = replies
        .iter()
        .position(|r| r.get("id") == Some(&json!("marker")))
        .ok_or("marker missing")?;
    if replies
        .iter()
        .skip(marker + 1)
        .any(|r| r.get("id") == Some(&json!("cancel-me")))
    {
        return Err("cancelled call wrote after notification marker".into());
    }
    if replies.len() > 2
        || serde_json::to_string(&replies)
            .map_err(|e| e.to_string())?
            .contains("SENSITIVE")
    {
        return Err("cancellation notification replied or echoed reason".into());
    }
    Ok(())
}

fn offline_observations() -> Check {
    let env = environment()?;
    let path = env.root().join("observations.jsonl");
    let globals = vec!["--observe".into(), path.to_string_lossy().into_owned()];
    let call = request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp", "arguments":{
        "args":["get", "gene", "BRAF", "--offline"]}}),
    );
    let replies = super::mcp::exchange_with_globals(&env, &[call], &globals)?;
    let outcome = reply(&replies, &json!(1))?;
    if outcome.pointer("/result/structuredContent/status") == Some(&json!("local")) {
        return Err(format!("offline tool reopened observations: {outcome}"));
    }
    if outcome
        .pointer("/result/structuredContent/sources/0")
        .is_none()
    {
        return Err(format!("offline tool did not execute: {outcome}"));
    }
    Ok(())
}
