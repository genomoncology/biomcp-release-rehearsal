//! Open selectors use existing public BRAF identity facts through recorded transport.
use super::super::{launch, mcp, network_stub::Stub};
use serde_json::{Value, json};

pub(super) fn enabled() -> Result<(), String> {
    let stub = Stub::for_source("myvariant.info", "application/json", body)?;
    let env = super::source_environment(&stub)?;
    let replay = env.root().join("recordings");
    let args = [
        "search",
        "variant",
        "--gene",
        "BRAF",
        "--protein",
        "V600E",
        "--record",
    ]
    .into_iter()
    .map(str::to_owned)
    .chain([replay.to_string_lossy().into()])
    .collect::<Vec<_>>();
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if value.pointer("/result/source_query")
        != Some(&json!("dbnsfp.genename:BRAF AND dbnsfp.hgvsp:\"p.V600E\""))
        || value.pointer("/result/selector_terms/usage") != Some(&json!("noncommercial"))
    {
        return Err(format!("enabled selector changed: {value}"));
    }
    super::super::network_stub::retain_replay_owner(env.shared_root().as_ref())
        .map_err(|e| e.to_string())?;
    let requests = stub.finish()?;
    if let Some(transcript) = super::super::transcript::current() {
        transcript.record(
            "cache/variant-search/enabled-protein-selector/requests",
            json!(requests),
        )?;
    }
    let [(_, request)] = requests.as_slice() else {
        return Err("enabled selector must send one source request".into());
    };
    if !request.starts_with("GET /v1/query?q=dbnsfp.genename%3ABRAF%20AND%20dbnsfp.hgvsp%3A%22p.V600E%22&size=10&from=0&fields=") {
        return Err(format!("enabled selector changed request bytes: {request}"));
    }
    Ok(())
}

pub(super) fn check() -> Result<(), String> {
    for protein in ["V600E", "Val600Glu", "p.V600E", "p.Val600Glu"] {
        check_policy(protein)?;
    }
    refusal()?;
    Ok(())
}

fn refusal() -> Result<(), String> {
    let env = launch::Environment::new().map_err(|e| e.to_string())?;
    let args = [
        "search",
        "variant",
        "--gene",
        "BRAF",
        "--protein",
        "V600E",
        "--disable-sources",
        "open,noncommercial",
        "--offline",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if value.pointer("/sources/0/reason") != Some(&json!("disabled")) {
        return Err(format!(
            "both disabled must keep the named refusal: {value}"
        ));
    }
    Ok(())
}

fn check_policy(protein: &str) -> Result<(), String> {
    let stub = Stub::for_source("myvariant.info", "application/json", body)?;
    let env = super::source_environment(&stub)?;
    let replay = env.root().join("recordings");
    let words = [
        "search",
        "variant",
        "--gene",
        "BRAF",
        "--protein",
        protein,
        "--disable-sources",
        "noncommercial",
    ];
    let mut args: Vec<String> = words.into_iter().map(str::to_owned).collect();
    args.extend(["--record".into(), replay.to_string_lossy().into()]);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let cli: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    check_reply(&cli)?;
    let replay_env = launch::Environment::new()
        .map_err(|e| e.to_string())?
        .with_replay_owner(env.shared_root());
    let args: Vec<String> = words.into_iter().map(str::to_owned).collect();
    let mut globals = replay_env.globals(Some(&replay), false);
    globals.push("--hosted".into());
    let hosted = mcp::call_with_globals(&replay_env, &args, false, &globals)?;
    check_reply(&hosted)?;
    hosted_http(&replay, protein)?;
    let requests = stub.finish()?;
    let [(_, request)] = requests.as_slice() else {
        return Err(format!("expected exactly one source request: {requests:?}"));
    };
    for forbidden in ["dbnsfp", "cadd", "cosmic", "snpedia"] {
        if request.to_lowercase().contains(forbidden) {
            return Err(format!("disabled component leaked: {forbidden}"));
        }
    }
    if !request.contains("snpeff.ann.genename%3ABRAF") || !request.contains("snpeff.ann.hgvs_p%3A")
    {
        return Err(format!("open fields missing from request: {request}"));
    }
    Ok(())
}

fn hosted_http(replay: &std::path::Path, protein: &str) -> Result<(), String> {
    let server = super::super::http::Server::configured_globals(
        "127.0.0.1:0",
        None,
        Some(replay.into()),
        None,
        None,
        &["--hosted".into()],
    )?;
    let frame = mcp::request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp", "arguments":{
        "args":["search", "variant", "--gene", "BRAF", "--protein", protein]}}),
    );
    let headers = "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\n\
        MCP-Protocol-Version: 2026-07-28\r\nMcp-Method: tools/call\r\nMcp-Name: biomcp\r\nx-biomcp-use: restricted\r\n";
    let (status, reply, _) = server.raw(
        "POST",
        "/mcp",
        &server.address.to_string(),
        headers,
        &frame.to_string(),
    )?;
    let reply = reply.ok_or("missing hosted reply")?;
    if status != 200 || reply.pointer("/result/isError") != Some(&json!(false)) {
        return Err(format!("restricted hosted request failed: {reply}"));
    }
    check_reply(
        reply
            .pointer("/result/structuredContent")
            .ok_or("missing hosted result")?,
    )
}

fn check_reply(value: &Value) -> Result<(), String> {
    for forbidden in ["dbnsfp", "cadd", "cosmic", "snpedia"] {
        if value.to_string().to_lowercase().contains(forbidden) {
            return Err(format!("disabled component leaked in reply: {forbidden}"));
        }
    }
    let term = concat!(
        "snpeff.ann.genename:BRAF AND snpeff.ann.hgvs_p:(",
        "\"V600E\" OR \"Val600Glu\" OR \"p.V600E\" OR \"p.Val600Glu\")"
    );
    if value.pointer("/result/source_query") != Some(&json!(term)) {
        return Err(format!(
            "open selector must search every accepted protein form: {value}"
        ));
    }
    if value.pointer("/result/items/0/record/id") != Some(&json!("chr7:g.140753336A>T"))
        || value.pointer("/result/selector_terms/name") != Some(&json!("SnpEff MIT"))
        || value.pointer("/result/items/0/transcripts/0/matched") != Some(&json!(true))
    {
        return Err(format!(
            "open protein selector did not return the recorded candidate: {value}"
        ));
    }
    Ok(())
}

fn body(_: &str) -> Result<String, String> {
    // Synthetic page and transcript projection of existing common.json BRAF facts;
    // no new assertion or current live-source result is claimed.
    Ok(json!({"total":1,"hits":[{"_id":"chr7:g.140753336A>T",
        "dbsnp":{"rsid":"rs113488022","gene":{"symbol":"BRAF"}},
        "clinvar":{"variant_id":13961},"snpeff":{"ann":[{
            "feature_id":"NM_004333.4","feature_type":"transcript","genename":"BRAF",
            "hgvs_c":"c.1799T>A","hgvs_p":"p.Val600Glu"}]}}]})
    .to_string())
}
