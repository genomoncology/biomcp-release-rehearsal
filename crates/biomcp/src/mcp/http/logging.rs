//! Operator diagnostics select only registry names and fixed statuses, never request text.
use serde_json::{Value, json};
use std::time::Duration;

pub(super) fn record(
    request: &super::wire::Request,
    duration: Duration,
    http: u16,
    reply: Option<&Value>,
) {
    if request.path != "/mcp" {
        return;
    }
    let Ok(value) = serde_json::from_slice::<Value>(&request.body) else {
        return;
    };
    if value.get("method").and_then(Value::as_str) != Some("tools/call") {
        return;
    }
    let mut line = line(
        value.get("params").unwrap_or(&Value::Null),
        duration,
        http,
        reply,
    )
    .to_string();
    line.push('\n');
    crate::output::diagnostic(line.as_bytes());
}
pub(super) fn line(params: &Value, duration: Duration, http: u16, reply: Option<&Value>) -> Value {
    let tool = params
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| {
            matches!(*name, "biomcp" | "biomcp_help" | "search" | "fetch")
                || crate::grammar::command_paths()
                    .iter()
                    .any(|(verb, entity, _)| *name == format!("{verb}_{entity}").replace('-', "_"))
        })
        .unwrap_or("unknown");
    let content = reply.and_then(|reply| {
        reply
            .pointer("/result/structuredContent")
            .cloned()
            .or_else(|| {
                let text = reply.pointer("/result/content/0/text")?.as_str()?;
                serde_json::from_str(text)
                    .ok()
                    .or_else(|| serde_json::from_str(text.rsplit_once('\n')?.1).ok())
            })
    });
    let evidence = content.as_ref().map(|value| {
        value
            .pointer("/metadata/envelope")
            .or_else(|| value.get("metadata"))
            .unwrap_or(value)
    });
    let status = evidence
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .filter(|status| {
            matches!(
                *status,
                "complete" | "partial" | "absent" | "failed" | "error"
            )
        })
        .unwrap_or_else(|| {
            if http != 200
                || reply.is_none_or(|value| {
                    value.get("error").is_some()
                        || value.pointer("/result/isError") == Some(&json!(true))
                })
            {
                "error"
            } else {
                "complete"
            }
        });
    let mut failed = evidence
        .and_then(|value| value.get("sources"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|source| source.get("state") == Some(&json!("failed")))
        .filter_map(|source| {
            serde_json::from_value::<biomcp_types::Source>(source.get("source")?.clone()).ok()
        })
        .filter_map(|source| serde_json::to_value(source).ok())
        .collect::<Vec<_>>();
    failed.sort_by_key(Value::to_string);
    failed.dedup();
    json!({"event":"mcp_call", "tool":tool, "duration_ms":duration.as_millis(), "status":status, "failed_sources":failed})
}
