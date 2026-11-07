#[path = "typed_descriptions.rs"]
mod descriptions;
// Both transports retain hosted policy while typed errors name executable next calls.
use super::super::super::{
    http::Server,
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};

pub(super) fn check_descriptions(env: &Environment) -> Result<(), String> {
    descriptions::check(env)
}

pub(super) fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed","hosted_max_use":"commercial"}))
        .map_err(|e| e.to_string())?;
    check_descriptions(&env)?;
    hosted(&env)?;
    boundaries(&env)
}
fn hosted(env: &Environment) -> Result<(), String> {
    let frames = frames();
    let replies = exchange_with_globals(env, &frames, &["--offline".into(), "--hosted".into()])?;
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"mcp_tool_mode":"typed","hosted_max_use":"commercial"})),
        None,
        None,
        None,
        &["--offline".into(), "--hosted".into()],
    )?;
    for (index, frame) in frames.iter().enumerate() {
        let stdio = replies
            .iter()
            .find(|r| r["id"] == frame["id"])
            .ok_or("stdio reply")?;
        let (status, http) = server.request("POST", "/mcp", &frame.to_string())?;
        if status != 200
            || stdio.pointer("/result/structuredContent")
                != http.pointer("/result/structuredContent")
        {
            return Err("typed hosted transport mismatch".into());
        }
        let text = stdio
            .pointer("/result/content/0/text")
            .and_then(Value::as_str)
            .ok_or("summary")?;
        if index < 5 {
            let command = if index == 1 || index == 2 {
                "get trial"
            } else {
                "get variant"
            };
            exact_next(
                env,
                text,
                json!({"name":"help","arguments":{"command":command}}),
            )?;
        }
        if index >= 5
            && text
                .lines()
                .next()
                .is_some_and(|line| line.contains("\"action\":\"search\""))
        {
            return Err(format!(
                "hosted refusal acquired a search suggestion: {text}"
            ));
        }
        if index >= 5
            && !stdio
                .pointer("/result/structuredContent")
                .ok_or("structured reply")?
                .to_string()
                .contains("hosted")
        {
            return Err("typed save escaped hosted refusal".into());
        }
    }
    get_help(env)?;
    fresh_refusal(env)
}
pub(super) fn get_help(env: &Environment) -> Result<(), String> {
    for (name, arguments, command) in [
        ("get_author", json!({"ids":["123"]}), "get author"),
        (
            "get_article",
            json!({"input":["secret-input","indexing"]}),
            "get article",
        ),
        (
            "get_article",
            json!({"input":["lung cancer"]}),
            "get article",
        ),
        ("get_trial", json!({"input":["NCT00000000"]}), "get trial"),
        ("get_variant", json!({"input":"--help"}), "get variant"),
        ("get_trial", json!({"input":["lung cancer"]}), "get trial"),
        ("get_variant", json!({"input":"BRAF V600E"}), "get variant"),
    ] {
        let frame = request(
            json!(10),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(name, arguments),
        );
        let replies = exchange_with_globals(env, &[frame], &["--offline".into()])?;
        let reply = replies.first().ok_or("invalid getter reply")?;
        if reply.pointer("/result/structuredContent/next/0/name") != Some(&json!("help"))
            || reply.pointer("/result/structuredContent/next/0/arguments/command")
                != Some(&json!(command))
        {
            return Err(format!("getter lost typed help: {reply}"));
        }
        let text = reply
            .pointer("/result/content/0/text")
            .and_then(Value::as_str)
            .ok_or("getter summary")?;
        exact_next(
            env,
            text,
            json!({"name":"help","arguments":{"command":command}}),
        )?;
    }
    Ok(())
}
fn fresh_refusal(env: &Environment) -> Result<(), String> {
    let frame = request(
        json!(9),
        "tools/call",
        crate::runner::protocol::typed_catalog::entity("get_gene", json!({"input":["BRAF"]})),
    );
    let replies = exchange_with_globals(env, &[frame], &["--hosted".into(), "--fresh".into()])?;
    let reply = replies.first().ok_or("hosted fresh reply")?;
    let text = reply
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .ok_or("summary")?;
    if reply.pointer("/result/structuredContent/error/code")
        != Some(&json!("usage.mcp_unavailable"))
        || text
            .lines()
            .next()
            .is_some_and(|line| line.contains("\"action\":\"search\""))
    {
        return Err(format!("hosted get refusal changed guidance: {reply}"));
    }
    Ok(())
}
fn boundaries(env: &Environment) -> Result<(), String> {
    let hostile = request(
        json!(8),
        "tools/call",
        crate::runner::protocol::typed_catalog::entity("get_gene", json!({"input":["--help"]})),
    );
    let reply = exchange_with_globals(env, &[hostile], &["--offline".into()])?;
    if reply
        .first()
        .ok_or("missing reply")?
        .pointer("/result/structuredContent/schema")
        == Some(&json!("biomcp.help/6"))
    {
        return Err("literal input became a CLI flag".into());
    }
    // A command-line mode explicitly overrides the configured mode.
    let reply = exchange_with_globals(
        env,
        &[request(json!(4), "tools/list", json!({}))],
        &["--tool-mode".into(), "single".into(), "--offline".into()],
    )?;
    if reply
        .first()
        .ok_or("missing reply")?
        .pointer("/result/tools/0/name")
        != Some(&json!("biomcp"))
    {
        return Err("flag did not override config".into());
    }
    Ok(())
}

fn exact_next(env: &Environment, text: &str, expected: Value) -> Result<(), String> {
    let first_line = text.lines().next().ok_or("typed summary missing")?;
    let (_, call) = first_line
        .split_once("next ")
        .ok_or("typed next call missing")?;
    let call: Value = serde_json::from_str(call).map_err(|e| e.to_string())?;
    if call != expected {
        return Err(format!("wrong CLI help translation: {call}"));
    }
    let replies = exchange_with_globals(
        env,
        &[request(json!(20), "tools/call", call)],
        &["--offline".into()],
    )?;
    let reply = replies.first().ok_or("next call reply missing")?;
    let status = reply
        .pointer("/result/structuredContent/status")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("next call rejected: {reply}"))?;
    if status != "complete"
        || reply.pointer("/result/structuredContent/schema") != Some(&json!("biomcp.help/6"))
    {
        return Err(format!("next call failed CLI validation: {reply}"));
    }
    Ok(())
}

fn frames() -> [Value; 6] {
    [
        request(
            json!(1),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(
                "get_variant",
                json!({"input":"BRAF V600E"}),
            ),
        ),
        request(
            json!(2),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(
                "get_trial",
                json!({"input":["melanoma"]}),
            ),
        ),
        request(
            json!(5),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(
                "get_trial",
                json!({"input":["lung cancer"]}),
            ),
        ),
        request(
            json!(6),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(
                "get_variant",
                json!({"input":"beta thalassemia"}),
            ),
        ),
        request(
            json!(7),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(
                "get_variant",
                json!({"input":"unrecognized"}),
            ),
        ),
        request(
            json!(3),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(
                "article_pdf",
                json!({"input":"28944238","save":"saved.pdf"}),
            ),
        ),
    ]
}
