//! Installed typed calls preserve CLI retirement boundaries and server freshness.
use super::super::{
    http::Server,
    launch::{Environment, arguments, fixtures},
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};

type Check = Result<(), String>;

pub(super) fn check() -> Check {
    for (argv, call) in [
        (
            vec!["get", "pathway", "hsa04151"],
            json!({"name":"pathway","arguments":{"action":"get","get":{"id":"hsa04151"}}}),
        ),
        (
            vec!["get", "pathway", "R-HSA-109581", "enrichment"],
            json!({"name":"pathway","arguments":{"action":"get","get":{"id":"R-HSA-109581","section":"enrichment"}}}),
        ),
        (
            vec![
                "get",
                "disease",
                "MONDO:0007254",
                "genes",
                "--source",
                "opentargets",
            ],
            crate::runner::protocol::typed_catalog::entity(
                "get_disease",
                json!({"id":"MONDO:0007254","section":"genes","source":"opentargets"}),
            ),
        ),
    ] {
        let env = environment()?;
        let globals = vec!["--offline".into()];
        let expected = cli(&env, &argv, &globals)?;
        if expected.pointer("/error/code") != Some(&json!("usage.retired")) {
            return Err(format!("missing CLI retirement: {expected}"));
        }
        parity(&env, &globals, call, &expected)?;
    }
    let env = environment()?;
    let globals = vec!["--offline".into()];
    let literal = cli(&env, &["get", "gene", "--", "--help"], &globals)?;
    if literal.get("schema") == Some(&json!("biomcp.help/6")) {
        return Err("literal CLI input became help".into());
    }
    parity(
        &env,
        &globals,
        crate::runner::protocol::typed_catalog::entity("get_gene", json!({"input":["--help"]})),
        &literal,
    )?;
    reference_record()?;
    fresh_cases()?;
    indication()
}

fn indication() -> Check {
    let env = environment()?;
    let replay = fixtures().join("replies/drug/indication");
    let globals = env.globals(Some(&replay), false);
    let expected = cli(
        &env,
        &[
            "search",
            "drug",
            "--indication",
            "melanoma",
            "--source",
            "openfda-label",
            "--limit",
            "1",
        ],
        &globals,
    )?;
    if expected.get("status") != Some(&json!("complete")) {
        return Err(format!("indication CLI search failed: {expected}"));
    }
    parity(
        &env,
        &globals,
        crate::runner::protocol::typed_catalog::entity(
            "search_drug",
            json!({"indication":"melanoma","source":"openfda-label","limit":1}),
        ),
        &expected,
    )
}

fn environment() -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}

fn parity(env: &Environment, globals: &[String], call: Value, expected: &Value) -> Check {
    let frame = request(json!(1), "tools/call", call);
    let actual = typed(env, globals, &frame)?;
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"mcp_tool_mode":"typed"})),
        None,
        None,
        None,
        globals,
    )?;
    let (status, http) = server.request("POST", "/mcp", &frame.to_string())?;
    if super::typed_followups::data(&actual) != super::typed_followups::data(expected)
        || status != 200
        || http
            .pointer("/result/structuredContent")
            .map(super::typed_followups::data)
            != Some(super::typed_followups::data(&actual))
    {
        return Err(format!(
            "typed bridge differs: CLI {expected}, stdio {actual}, HTTP {http}"
        ));
    }
    Ok(())
}

fn reference_record() -> Check {
    let env = environment()?;
    let replay = fixtures().join("replies/pathway/native");
    let globals = env.globals(Some(&replay), false);
    let expected = cli(&env, &["get", "pathway", "R-HSA-5673001"], &globals)?;
    if expected.get("status") != Some(&json!("complete")) {
        return Err(format!("recorded pathway fixture failed: {expected}"));
    }
    parity(
        &env,
        &globals,
        json!({"name":"pathway","arguments":{"action":"get","get":{"id":"R-HSA-5673001"}}}),
        &expected,
    )
}

fn fresh_cases() -> Check {
    for (source, argv, call) in [
        (
            "ema",
            vec![
                "search", "drug", "Tibsovo", "--source", "ema", "--limit", "1", "--offset", "1",
            ],
            crate::runner::protocol::typed_catalog::entity(
                "search_drug",
                json!({"query":"Tibsovo","source":"ema","limit":1,"offset":1}),
            ),
        ),
        (
            "ddinter",
            vec![
                "get",
                "drug",
                "DDInter1",
                "interactions",
                "--source",
                "ddinter",
                "--section-limit",
                "1",
                "--section-offset",
                "1",
            ],
            crate::runner::protocol::typed_catalog::entity(
                "get_drug",
                json!({"input":"DDInter1","section":["interactions"],"source":"ddinter","section_limit":1,"section_offset":1}),
            ),
        ),
    ] {
        fresh(source, &argv, call)?;
    }
    Ok(())
}

fn fresh(source: &str, argv: &[&str], call: Value) -> Check {
    let env = environment()?;
    let replay = fixtures()
        .join("replies/drug")
        .join(format!("{source}-synthetic"));
    let mut globals = env.globals(Some(&replay), false);
    let synced = cli(&env, &["sync", source], &globals)?;
    if synced.get("status") != Some(&json!("complete")) {
        return Err(format!("fixture sync failed: {synced}"));
    }
    globals.push("--fresh".into());
    let expected = cli(&env, argv, &globals)?;
    if expected.get("status") != Some(&json!("complete"))
        || !expected
            .pointer("/next/0/args")
            .and_then(Value::as_array)
            .is_some_and(|args| args.contains(&json!("--fresh")))
    {
        return Err(format!("CLI fresh fixture lost continuation: {expected}"));
    }
    let frame = request(json!(1), "tools/call", call);
    let actual = typed(&env, &globals, &frame)?;
    if super::typed_followups::data(&actual) != super::typed_followups::data(&expected) {
        return Err(format!(
            "server freshness lost: CLI {expected}, typed {actual}"
        ));
    }
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"mcp_tool_mode":"typed"})),
        None,
        None,
        None,
        &globals,
    )?;
    let (status, http) = server.request("POST", "/mcp", &frame.to_string())?;
    if status != 200
        || http.pointer("/result/isError") != Some(&json!(true))
        || http.pointer("/result/structuredContent/status") != Some(&json!("usage"))
    {
        return Err(format!("HTTP fresh guard changed: {http}"));
    }
    Ok(())
}

fn typed(env: &Environment, globals: &[String], frame: &Value) -> Result<Value, String> {
    let replies = exchange_with_globals(env, std::slice::from_ref(frame), globals)?;
    replies
        .first()
        .and_then(|reply| reply.pointer("/result/structuredContent"))
        .cloned()
        .ok_or_else(|| format!("missing typed envelope: {replies:?}"))
}

fn cli(env: &Environment, argv: &[&str], globals: &[String]) -> Result<Value, String> {
    let argv = argv
        .iter()
        .map(|word| (*word).into())
        .collect::<Vec<String>>();
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &arguments(&argv, globals))
        .output()
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&output.stdout).map_err(|e| format!("CLI envelope: {e}"))
}
