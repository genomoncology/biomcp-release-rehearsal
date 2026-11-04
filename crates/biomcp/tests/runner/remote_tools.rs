//! Features mcp-http and mcp-parity: remote metadata and complete tool-result limits.
use super::{http::Server, launch::Environment, mcp::request};
use crate::runner::transcript::trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial("mcp-http/tool-dialects-and-transport-annotations", || {
            metadata().map_err(Into::into)
        }),
        trial("mcp-parity/complete-tool-result-character-cap", || {
            results().map_err(Into::into)
        }),
    ]
}
fn metadata() -> Result<(), String> {
    let server = Server::start()?;
    let (_, reply) = server.request(
        "POST",
        "/mcp",
        &request(json!(1), "tools/list", json!({})).to_string(),
    )?;
    check_tools(&reply, true)?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replies = super::mcp::exchange(&env, &[request(json!(1), "tools/list", json!({}))])?;
    check_tools(replies.first().ok_or("missing stdio tools")?, false)?;
    let hosted = super::mcp::exchange_with_globals(
        &env,
        &[request(json!(1), "tools/list", json!({}))],
        &["--hosted".into()],
    )?;
    check_tools(hosted.first().ok_or("missing hosted stdio tools")?, true)
}
fn check_tools(reply: &Value, read_only: bool) -> Result<(), String> {
    let tools = reply
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .ok_or("missing tools")?;
    if tools.len() != 3 {
        return Err("remote tool set changed".into());
    }
    for tool in tools {
        for key in ["inputSchema", "outputSchema"] {
            let schema = tool.get(key).ok_or("missing tool schema")?;
            if schema.get("$schema") != Some(&json!("https://json-schema.org/draft/2020-12/schema"))
            {
                return Err(format!("{} lacks declared dialect", key));
            }
            jsonschema::validator_for(schema).map_err(|e| e.to_string())?;
        }
        let expected = read_only || tool.get("name") != Some(&json!("biomcp"));
        if tool.pointer("/annotations/readOnlyHint") != Some(&json!(expected))
            || tool.pointer("/annotations/destructiveHint") != Some(&json!(false))
            || tool.pointer("/annotations/openWorldHint") != Some(&json!(true))
        {
            return Err("tool annotations disagree with save permissions".into());
        }
    }
    Ok(())
}
fn results() -> Result<(), String> {
    // Unicode remains below the character cap despite exceeding it in UTF-8 bytes.
    // ASCII crosses only when both content forms are included. Quotes exercise JSON escaping.
    for (text, over) in [
        ("é🙂".repeat(25_000), false),
        ("x".repeat(76_000), true),
        ("\"\\".repeat(14_000), true),
    ] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let recordings = env.root().join("recordings");
        super::response_cases::copy_capture("gene/main", &recordings, |source, record| {
            if source != "mygene" {
                return;
            }
            if let Some(exchanges) = record.get_mut("exchanges").and_then(Value::as_array_mut) {
                for exchange in exchanges {
                    if let Some(rows) = exchange
                        .pointer_mut("/body/value")
                        .and_then(Value::as_array_mut)
                    {
                        for row in rows {
                            row["summary"] = json!(text);
                        }
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;
        let server = Server::start_replay(recordings)?;
        let (_, reply) = server.request(
            "POST",
            "/mcp",
            &request(
                json!(2),
                "tools/call",
                json!({"name":"biomcp","arguments":{"args":["get","gene","BRAF"]}}),
            )
            .to_string(),
        )?;
        check_result(&reply, &text, over)?;
    }
    Ok(())
}

fn check_result(reply: &Value, text: &str, over: bool) -> Result<(), String> {
    let result = reply.get("result").ok_or("missing result")?;
    let serialized = result.to_string();
    if serialized.chars().count() >= super::output_limit("remote-result-characters")?
        || result.get("isError") != Some(&json!(over))
    {
        return Err(format!(
            "whole result cap or error flag: {} characters",
            serialized.chars().count()
        ));
    }
    let structured = result
        .get("structuredContent")
        .ok_or("missing structured content")?;
    let content: Value = serde_json::from_str(
        result
            .pointer("/content/0/text")
            .and_then(Value::as_str)
            .ok_or("missing text")?,
    )
    .map_err(|e| e.to_string())?;
    if content != *structured {
        return Err("tool text differs from structured result".into());
    }
    let error_schema: Value =
        serde_json::from_str(include_str!("../../../biomcp-types/schemas/error.json"))
            .map_err(|e| e.to_string())?;
    if over {
        jsonschema::validator_for(&error_schema)
            .map_err(|e| e.to_string())?
            .validate(structured)
            .map_err(|e| e.to_string())?;
        if structured.pointer("/error/code") != Some(&json!("usage.too_broad"))
            || structured.pointer("/error/count") != Some(&json!(150_000))
        {
            return Err("over-cap reply lacks typed cap error".into());
        }
        let message = structured
            .pointer("/error/message")
            .and_then(Value::as_str)
            .ok_or("missing guidance")?;
        if ![
            "character cap",
            "--limit",
            "--section-limit",
            "source",
            "section",
        ]
        .iter()
        .all(|word| message.contains(word))
        {
            return Err("over-cap reply lacks narrowing guidance".into());
        }
    } else if serialized.len() <= super::output_limit("remote-result-characters")?
        || !serialized.contains(text)
    {
        return Err("Unicode reply was cut or measured in bytes".into());
    }
    Ok(())
}
