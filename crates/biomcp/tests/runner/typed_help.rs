//! Installed typed help routes preserve CLI guidance and research contracts.
use super::super::{
    http::Server,
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};

pub(super) fn discovery() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let single = frames(
        &env,
        "single",
        &[
            request(json!(1), "tools/list", json!({})),
            request(
                json!(2),
                "tools/call",
                json!({"name":"search","arguments":{"query":"trial --condition melanoma"}}),
            ),
            request(
                json!(3),
                "tools/call",
                json!({"name":"fetch","arguments":{"id":"trial NCT04303780"}}),
            ),
            request(
                json!(4),
                "tools/call",
                json!({"name":"biomcp","arguments":{"args":["help","search","trial"]}}),
            ),
            request(json!(5), "server/discover", json!({})),
        ],
    )?;
    let typed = frames(
        &env,
        "typed",
        &[
            request(json!(1), "tools/list", json!({})),
            request(
                json!(2),
                "tools/call",
                json!({"name":"search","arguments":{"query":"trial --condition melanoma"}}),
            ),
            request(
                json!(3),
                "tools/call",
                json!({"name":"fetch","arguments":{"id":"trial NCT04303780"}}),
            ),
            request(
                json!(4),
                "tools/call",
                json!({"name":"biomcp_help","arguments":{"command":"search trial"}}),
            ),
            request(json!(5), "server/discover", json!({})),
        ],
    )?;
    discovery_contract(&single, &typed)
}
fn discovery_contract(single: &[Value], typed: &[Value]) -> Result<(), String> {
    let tools = reply(typed, 1)?
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .ok_or("typed catalog")?;
    super::typed_catalog::check(tools)?;
    let original = reply(single, 1)?
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .ok_or("single catalog")?;
    for name in ["search", "fetch"] {
        let expected = original
            .iter()
            .find(|t| t.get("name") == Some(&json!(name)))
            .ok_or("research tool")?;
        if !tools.contains(expected) {
            return Err(format!("{name} contract changed"));
        }
    }
    for id in [2, 3, 4] {
        let expected = reply(single, id)?
            .pointer("/result/structuredContent")
            .ok_or("single result")?;
        if super::typed_followups::data(
            reply(typed, id)?
                .pointer("/result/structuredContent")
                .ok_or("typed result")?,
        ) != super::typed_followups::data(expected)
        {
            return Err(format!("typed parity {id}"));
        }
    }
    if reply(typed, 4)?.pointer("/result/isError") != Some(&json!(false)) {
        return Err("help call refused".into());
    }
    for replies in [single, typed] {
        let guidance = reply(replies, 5)?
            .pointer("/result/instructions")
            .and_then(Value::as_str)
            .ok_or("instructions")?;
        if guidance
            .matches("Listed choices accept any letter case")
            .count()
            != 1
            || !guidance.contains("replies use their listed spelling")
        {
            return Err("case guidance must appear once in discovery instructions".into());
        }
    }
    let instructions = reply(typed, 5)?
        .pointer("/result/instructions")
        .and_then(Value::as_str)
        .ok_or("instructions")?;
    if instructions.chars().count() > 4096
        || !instructions
            .lines()
            .next()
            .is_some_and(|line| line.contains("biomcp_help"))
        || instructions.contains("help sources")
    {
        return Err("typed instructions lack useful help or exceed budget".into());
    }
    for (verb, entity, _) in biomcp_types::command_paths() {
        let name = format!("{verb}_{entity}").replace('-', "_");
        if !instructions
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .any(|word| word == name)
        {
            return Err(format!("map missing {name}"));
        }
    }
    Ok(())
}
fn frames(env: &Environment, mode: &str, calls: &[Value]) -> Result<Vec<Value>, String> {
    env.config(&json!({"mcp_tool_mode":mode}))
        .map_err(|e| e.to_string())?;
    exchange_with_globals(env, calls, &["--offline".into()])
}
fn reply(replies: &[Value], id: u64) -> Result<&Value, String> {
    replies
        .iter()
        .find(|r| r.get("id") == Some(&json!(id)))
        .ok_or_else(|| format!("missing reply {id}"))
}
pub(super) fn recovery() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    let globals = ["--offline".into()];
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"mcp_tool_mode":"typed"})),
        None,
        None,
        None,
        &globals,
    )?;
    for (name, arguments, command) in [
        (
            "search_trial",
            json!({"limit":"bad","source":"bad"}),
            "search trial",
        ),
        ("get_gene", json!({}), "get gene"),
    ] {
        let initial = request(
            json!(1),
            "tools/call",
            json!({"name":name,"arguments":arguments}),
        );
        for http in [false, true] {
            let result = call(&env, &server, &initial, http)?;
            if result.pointer("/result/isError") != Some(&json!(true))
                || result.pointer("/result/structuredContent/status") != Some(&json!("usage"))
            {
                return Err(format!("invalid call lost CLI usage: {result}"));
            }
            let text = result
                .pointer("/result/content/0/text")
                .and_then(Value::as_str)
                .ok_or("summary")?;
            let next = text
                .lines()
                .next()
                .and_then(|line| line.split_once("next "))
                .map(|(_, next)| next)
                .ok_or_else(|| format!("no typed help next: {text}"))?;
            let next: Value = serde_json::from_str(next).map_err(|e| e.to_string())?;
            let expected = json!({"name":"biomcp_help","arguments":{"command":command}});
            if next != expected {
                return Err(format!("wrong help next: {next}, expected {expected}"));
            }
            let executed = call(&env, &server, &request(json!(2), "tools/call", next), http)?;
            if executed.pointer("/result/isError") != Some(&json!(false))
                || executed.pointer("/result/structuredContent/schema")
                    != Some(&json!("biomcp.help/6"))
            {
                return Err(format!("help next not executable: {executed}"));
            }
        }
    }
    Ok(())
}
fn call(env: &Environment, server: &Server, frame: &Value, http: bool) -> Result<Value, String> {
    if http {
        let (status, result) = server.request("POST", "/mcp", &frame.to_string())?;
        if status != 200 {
            return Err(format!("HTTP {status}: {result}"));
        }
        Ok(result)
    } else {
        exchange_with_globals(env, std::slice::from_ref(frame), &["--offline".into()])?
            .into_iter()
            .next()
            .ok_or("stdio reply".into())
    }
}
