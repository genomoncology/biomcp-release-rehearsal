//! Installed typed replies preserve configured representations and the JSON-value cap.
use super::super::{
    http::Server,
    launch::{self, Environment},
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};

pub(super) fn check() -> Result<(), String> {
    forms()?;
    notes()?;
    cap()
}
fn forms() -> Result<(), String> {
    let capture = launch::fixtures().join("replies/trial/condition");
    let mut reference = None;
    for form in [
        None,
        Some("both"),
        Some("structured-only"),
        Some("text-only"),
    ] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let mut config = json!({"mcp_tool_mode":"typed"});
        if let Some(form) = form {
            config
                .as_object_mut()
                .ok_or("config object")?
                .insert("mcp_reply_form".into(), json!(form));
        }
        env.config(&config).map_err(|e| e.to_string())?;
        let frames = [
            request(
                json!(1),
                "tools/call",
                crate::runner::protocol::typed_catalog::entity(
                    "search_trial",
                    json!({"condition":["melanoma"]}),
                ),
            ),
            request(
                json!(2),
                "tools/call",
                crate::runner::protocol::typed_catalog::entity(
                    "get_trial",
                    json!({"input":["not-an-nct"]}),
                ),
            ),
            request(
                json!(3),
                "tools/call",
                crate::runner::protocol::typed_catalog::entity(
                    "get_trial",
                    json!({"input":["NCT07491445"],"section":["summary","eligibility"],"source":"ctgov"}),
                ),
            ),
        ];
        let replies = exchange_with_globals(&env, &frames, &env.globals(Some(&capture), false))?;
        let values = frames
            .iter()
            .enumerate()
            .map(|(index, frame)| {
                let result = replies
                    .iter()
                    .find(|reply| reply.get("id") == frame.get("id"))
                    .and_then(|reply| reply.get("result"))
                    .ok_or("tool result missing")?;
                checked(result, form.unwrap_or("both"), index)
            })
            .collect::<Result<Vec<_>, String>>()?;
        http_forms(config, &capture, &frames, form.unwrap_or("both"))?;
        if let Some(reference) = &reference {
            if reference != &values {
                return Err("reply form changed typed JSON".into());
            }
        } else {
            reference = Some(values);
        }
    }
    Ok(())
}
// A call with no next step and an oversized reply both end in the form's own note.
fn notes() -> Result<(), String> {
    for (form, note) in [
        (None, "JSON reply follows."),
        (Some("both"), "JSON reply follows."),
        (Some("text-only"), "JSON reply follows."),
        (
            Some("structured-only"),
            "JSON reply is in structuredContent.",
        ),
    ] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let mut config = json!({"mcp_tool_mode":"typed"});
        if let Some(form) = form {
            config
                .as_object_mut()
                .ok_or("config object")?
                .insert("mcp_reply_form".into(), json!(form));
        }
        env.config(&config).map_err(|e| e.to_string())?;
        let recordings = super::super::remote_tools::recording(&env, &"x".repeat(151_000))?;
        let frames = [
            request(
                json!(1),
                "tools/call",
                json!({"name":"help","arguments":{"command":"get trial"}}),
            ),
            request(
                json!(2),
                "tools/call",
                crate::runner::protocol::typed_catalog::entity(
                    "get_gene",
                    json!({"input":["BRAF"]}),
                ),
            ),
        ];
        let replies = exchange_with_globals(&env, &frames, &env.globals(Some(&recordings), false))?;
        let server = Server::configured_globals(
            "127.0.0.1:0",
            Some(config),
            Some(recordings),
            None,
            None,
            &[],
        )?;
        for frame in &frames {
            let stdio = replies
                .iter()
                .find(|reply| reply.get("id") == frame.get("id"))
                .ok_or("stdio reply missing")?;
            let (status, http) = server.request("POST", "/mcp", &frame.to_string())?;
            if status != 200 {
                return Err("HTTP typed call failed".into());
            }
            for reply in [stdio, &http] {
                let result = reply.get("result").ok_or("tool result missing")?;
                let value = represented(result, form.unwrap_or("both"))?;
                let expected = format!(
                    "{}: {note}",
                    value
                        .get("status")
                        .and_then(Value::as_str)
                        .ok_or("status missing")?
                );
                let text = result
                    .pointer("/content/0/text")
                    .and_then(Value::as_str)
                    .ok_or("text missing")?;
                if text.lines().next() != Some(expected.as_str()) {
                    return Err(format!("expected {expected:?} first: {text:.200}"));
                }
            }
        }
    }
    Ok(())
}
fn http_forms(
    config: Value,
    capture: &std::path::Path,
    frames: &[Value],
    form: &str,
) -> Result<(), String> {
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(config),
        Some(capture.to_owned()),
        None,
        None,
        &[],
    )?;
    for (index, frame) in frames.iter().enumerate() {
        let (status, reply) = server.request("POST", "/mcp", &frame.to_string())?;
        if status != 200 {
            return Err("HTTP typed call failed".into());
        }
        checked(
            reply.get("result").ok_or("HTTP result missing")?,
            form,
            index,
        )?;
    }
    Ok(())
}
fn checked(result: &Value, form: &str, index: usize) -> Result<Value, String> {
    let value = represented(result, form)?;
    if result.get("isError") != Some(&json!(index != 0)) {
        return Err("typed reply lost error status".into());
    }
    if index == 0 && value.get("status") != Some(&json!("complete")) {
        return Err(format!("recorded search did not succeed: {value}"));
    }
    if index == 2
        && value.get("next")
            != Some(&json!([
                {"kind":"configure","name":"trial","arguments":{"action":"get","get":{"input":["NCT07491445"],"section":["eligibility"]}}},
                {"kind":"configure","name":"help","arguments":{"command":"get trial"}}
            ]))
    {
        return Err(format!("typed reply lost ordered recovery steps: {value}"));
    }
    Ok(value)
}
fn represented(result: &Value, form: &str) -> Result<Value, String> {
    let text = result
        .pointer("/content/0/text")
        .and_then(Value::as_str)
        .ok_or("text missing")?;
    let first = text.lines().next().ok_or("summary missing")?;
    if !first.contains(':') || first.starts_with('{') {
        return Err("typed summary must come first".into());
    }
    if form != "structured-only" && first.contains("structuredContent") {
        return Err(format!("text reply points at structuredContent: {first}"));
    }
    let value = if form == "structured-only" {
        if text.lines().count() != 1 {
            return Err("structured-only repeats JSON text".into());
        }
        result
            .get("structuredContent")
            .cloned()
            .ok_or("structured reply missing")?
    } else {
        let (_, payload) = text
            .split_once('\n')
            .ok_or("serialized JSON missing after summary")?;
        let value: Value = serde_json::from_str(payload).map_err(|e| e.to_string())?;
        if form == "text-only" {
            if result.get("structuredContent").is_some() {
                return Err("text-only repeats structured JSON".into());
            }
        } else if result.get("structuredContent") != Some(&value) {
            return Err("typed JSON copies differ".into());
        }
        value
    };
    if !first.starts_with(
        value
            .get("status")
            .and_then(Value::as_str)
            .ok_or("status missing")?,
    ) {
        return Err("summary status differs from JSON".into());
    }
    Ok(value)
}
fn cap() -> Result<(), String> {
    for (length, refused) in [(80_000, false), (151_000, true)] {
        let text = "x".repeat(length);
        let mut reference = None;
        for mode in ["single", "typed"] {
            let env = Environment::new().map_err(|e| e.to_string())?;
            env.config(&json!({"mcp_tool_mode":mode}))
                .map_err(|e| e.to_string())?;
            let recordings = super::super::remote_tools::recording(&env, &text)?;
            let params = if mode == "single" {
                json!({"name":"biomcp","arguments":{"args":["get","gene","BRAF"]}})
            } else {
                crate::runner::protocol::typed_catalog::entity(
                    "get_gene",
                    json!({"input":["BRAF"]}),
                )
            };
            let replies = exchange_with_globals(
                &env,
                &[request(json!(1), "tools/call", params)],
                &env.globals(Some(&recordings), false),
            )?;
            let value = replies
                .first()
                .and_then(|reply| reply.pointer("/result/structuredContent"))
                .ok_or("cap reply missing")?;
            let broad = value.pointer("/error/code") == Some(&json!("usage.too_broad"));
            if broad
                && mode == "typed"
                && replies
                    .first()
                    .and_then(|reply| reply.pointer("/result/content/0/text"))
                    .and_then(Value::as_str)
                    .and_then(|text| text.lines().next())
                    .is_some_and(|text| text.contains("\"action\":\"search\""))
            {
                return Err("too-broad reply acquired a search suggestion".into());
            }
            if broad != refused {
                return Err(format!(
                    "{mode} cap differed at recorded summary length {length}: {}",
                    value.get("error").unwrap_or(&Value::Null)
                ));
            }
            if let Some(reference) = &reference {
                if super::typed_followups::data(reference) != super::typed_followups::data(value) {
                    return Err("single and typed cap replies differ".into());
                }
            } else {
                reference = Some(value.clone());
            }
        }
    }
    Ok(())
}
