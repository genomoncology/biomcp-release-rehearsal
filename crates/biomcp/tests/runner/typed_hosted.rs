//! Discovery describes the same file and source authority as execution.
use super::super::{
    http::Server,
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};
use std::io::Write;

pub(super) fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    let mut budgets = Vec::new();
    let frames = [request(json!(1), "tools/list", json!({}))];
    let local = exchange_with_globals(&env, &frames, &["--offline".into()])?;
    capture("local", local.first().ok_or("local reply")?, &mut budgets)?;
    let tools = local
        .first()
        .and_then(|r| r.pointer("/result/tools"))
        .and_then(Value::as_array)
        .ok_or("local tools")?;
    for tool in tools {
        let name = tool
            .get("name")
            .and_then(Value::as_str)
            .ok_or("tool name")?;
        let writes = matches!(name, "biomcp" | "biomcp_reference");
        if tool.pointer("/annotations/readOnlyHint") != Some(&json!(!writes)) {
            return Err(format!("wrong file capability hint: {name}"));
        }
    }
    configured_exclusions(&env, &mut budgets)?;
    hosted(&mut budgets)?;
    if budgets.is_empty() {
        Ok(())
    } else {
        Err(budgets.join("\n"))
    }
}

fn hosted(budgets: &mut Vec<String>) -> Result<(), String> {
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"mcp_tool_mode":"typed"})),
        None,
        None,
        None,
        &["--offline".into(), "--hosted".into()],
    )?;
    let hosted = catalog(&server, None, budgets)?;
    capture("restricted", &hosted, budgets)?;
    assert_hosted(&hosted)?;
    refused_source(&server)?;
    for usage in ["commercial", "academic"] {
        let reply = catalog(&server, Some(usage), budgets)?;
        capture(usage, &reply, budgets)?;
        let expected = usage == "academic";
        let choices = reply
            .pointer("/result/tools")
            .and_then(Value::as_array)
            .and_then(|tools| {
                tools
                    .iter()
                    .find(|tool| tool.get("name") == Some(&json!("search_drug")))
            })
            .and_then(|tool| tool.pointer("/inputSchema/properties/source/enum"))
            .and_then(Value::as_array)
            .ok_or("drug source choices")?;
        if choices.contains(&json!("who-pq")) != expected {
            return Err(format!(
                "{usage} source choices disagree with WHO source policy"
            ));
        }
    }
    let call = request(
        json!(2),
        "tools/call",
        json!({"name":"article_pdf","arguments":{"input":"28944238","save":"saved.pdf"}}),
    );
    let (_, reply) = server.request("POST", "/mcp", &call.to_string())?;
    if reply.pointer("/result/structuredContent/error/code")
        != Some(&json!("usage.mcp_unavailable"))
    {
        return Err(format!("omitted tool escaped execution policy: {reply}"));
    }
    Ok(())
}

fn assert_hosted(hosted: &Value) -> Result<(), String> {
    let tools = hosted
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .ok_or("hosted tools")?;
    for tool in tools {
        if matches!(
            tool.get("name").and_then(Value::as_str),
            Some("article_pdf" | "trial_document")
        ) {
            return Err("hosted catalog advertises mandatory save".into());
        }
        if tool.pointer("/annotations/readOnlyHint") != Some(&json!(true)) {
            return Err("hosted catalog marks a refused write as writable".into());
        }
        let advertised =
            json!({"description":tool.get("description"),"inputSchema":tool.get("inputSchema")})
                .to_string();
        if advertised.to_lowercase().contains("oncokb")
            || advertised.contains("who-pq")
            || advertised
                .to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .any(|word| word == "who")
        {
            return Err(format!(
                "restricted catalog advertises refused source: {}",
                tool.get("name").unwrap_or(&Value::Null)
            ));
        }
    }
    Ok(())
}

fn catalog(
    server: &Server,
    usage: Option<&str>,
    budgets: &mut Vec<String>,
) -> Result<Value, String> {
    let frame = request(json!(1), "tools/list", json!({}));
    let usage = usage
        .map(|usage| format!("x-biomcp-use: {usage}\r\n"))
        .unwrap_or_default();
    let headers = format!(
        "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2026-07-28\r\nMcp-Method: tools/list\r\n{usage}"
    );
    let (status, reply, _) = server.raw(
        "POST",
        "/mcp",
        &server.address.to_string(),
        &headers,
        &frame.to_string(),
    )?;
    if status != 200 {
        return Err(format!("catalog status {status}"));
    }
    let reply = reply.ok_or("catalog reply")?;
    let frame = request(json!(3), "server/discover", json!({}));
    let (_, discovery, _) = server.raw(
        "POST",
        "/mcp",
        &server.address.to_string(),
        &headers.replace("Mcp-Method: tools/list", "Mcp-Method: server/discover"),
        &frame.to_string(),
    )?;
    let discovery = discovery.ok_or("discovery reply")?;
    let instructions = discovery
        .pointer("/result/instructions")
        .and_then(Value::as_str)
        .ok_or("instructions")?;
    if instructions.chars().count() > 4096
        || usage.is_empty()
            && (instructions.contains("oncokb")
                || instructions.contains("article_pdf")
                || instructions.contains("trial_document"))
    {
        return Err("hosted instructions advertise refused tools/sources or exceed budget".into());
    }
    capture(&format!("instructions-{usage}"), &discovery, budgets)?;
    Ok(reply)
}

fn capture(label: &str, reply: &Value, budgets: &mut Vec<String>) -> Result<(), String> {
    if !label.starts_with("instructions-") {
        let tools = reply
            .pointer("/result/tools")
            .and_then(Value::as_array)
            .filter(|tools| !tools.is_empty())
            .ok_or_else(|| format!("{label} catalog has no tools: {reply}"))?;
        for tool in tools {
            writeln!(
                std::io::stdout(),
                "TYPED_CATALOG_SIZE {label} {} characters={}",
                tool.get("name").unwrap_or(&Value::Null),
                super::typed_schemas::size(tool)
            )
            .map_err(|e| e.to_string())?;
            if let Err(error) = super::typed_schemas::budget(tool) {
                budgets.push(format!("{label}: {error}"));
            }
        }
    }
    if let Some(transcript) = super::super::transcript::current() {
        transcript.record(&format!("typed-catalog/{label}"), reply.clone())?;
    }
    Ok(())
}

fn refused_source(server: &Server) -> Result<(), String> {
    let call = request(
        json!(4),
        "tools/call",
        json!({"name":"search_drug","arguments":{"query":"aspirin","source":"who-pq"}}),
    );
    let (_, reply) = server.request("POST", "/mcp", &call.to_string())?;
    let disabled = reply
        .pointer("/result/structuredContent/sources")
        .and_then(Value::as_array)
        .is_some_and(|sources| {
            sources.iter().any(|source| {
                source.get("source") == Some(&json!("who-pq"))
                    && source.get("reason") == Some(&json!("disabled"))
            })
        });
    if !disabled {
        return Err(format!(
            "catalog source exclusion differs from execution: {reply}"
        ));
    }
    Ok(())
}

fn configured_exclusions(env: &Environment, budgets: &mut Vec<String>) -> Result<(), String> {
    env.config(&json!({"mcp_tool_mode":"typed","disable_sources":["ctgov","who-pq","gwas"]}))
        .map_err(|e| e.to_string())?;
    let replies = exchange_with_globals(
        env,
        &[request(json!(1), "tools/list", json!({}))],
        &["--offline".into()],
    )?;
    let reply = replies.first().ok_or("configured catalog")?;
    capture("local-excluded", reply, budgets)?;
    if !reply.pointer("/result/tools").is_some_and(Value::is_array) {
        return Err("configured catalog missing tools".into());
    }
    if reply
        .to_string()
        .to_lowercase()
        .contains("clinicaltrials.gov")
    {
        return Err("excluded source display name remains in argument prose".into());
    }
    let marker = reply
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .and_then(|tools| {
            tools
                .iter()
                .find(|tool| tool.get("name") == Some(&json!("search_diagnostic")))
        })
        .and_then(|tool| tool.pointer("/inputSchema/properties/marker/description"));
    let reference = reply
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .and_then(|tools| tools.iter().find(|tool| tool["name"] == "biomcp_reference"))
        .and_then(|tool| tool.pointer("/inputSchema/properties/command/enum"))
        .and_then(Value::as_array)
        .ok_or("reference commands")?;
    if !reference.contains(&json!("search_gwas")) {
        return Err("source exclusion removed the reference command".into());
    }
    if marker != Some(&json!("WHO marker phrase.")) {
        return Err("unrelated publisher exclusion hid allowed WHO diagnostic prose".into());
    }
    Ok(())
}
