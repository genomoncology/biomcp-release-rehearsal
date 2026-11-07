//! Feature gene-bundle: a hosted server answers from its stored copy past the
//! refresh date with that copy's date, while the CLI keeps it partial.
use super::{
    bundle_wait::native_body, configuration::invoke, launch::Environment, mcp::request,
    network_stub::Stub, publication::Peer,
};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;

// 2026-01-01T00:00:00Z, so the stated date never follows the calendar.
const RETRIEVED: i64 = 1_767_225_600;

pub(super) fn trials() -> Vec<Trial> {
    vec![super::transcript::trial(
        "gene-bundle/hosted-past-refresh-states-date",
        || read().map_err(Into::into),
    )]
}

fn read() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = super::launch::fixtures().join("replies/gene/gencc");
    let body = native_body(&replay, "hgnc")?;
    let stub = Stub::for_sources(
        &["storage.googleapis.com"],
        |_| "text/tab-separated-values",
        move |_| Ok(body.clone()),
    )?;
    stub.retain(env.shared_root());
    let ca = env.root().join("ca.pem");
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(
        &json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,
        "disable_sources":["oncokb"]}),
    )
    .map_err(|e| e.to_string())?;
    invoke(&env, args(&["get", "gene", "ODC1"]), 0)?;
    expire(&env)?;
    let cli = invoke(&env, args(&["--offline", "get", "gene", "ODC1"]), 0)?;
    let local = checked(&cli, "partial", "stale")?;
    if notice(local).is_some() {
        return Err(format!("the CLI stated a hosted copy date: {local}"));
    }
    let mut server = Peer::with_globals(&env, &["--hosted".into()])?;
    server.send(&request(
        json!("hosted"),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":["get","gene","ODC1"]}}),
    ))?;
    let reply = server.next()?;
    let result = reply
        .pointer("/result/structuredContent")
        .ok_or_else(|| format!("missing content: {reply}"))
        .and_then(|value| checked(value, "complete", "ok").map(notice))
        .and_then(|found| match found {
            Some(message) if message.contains("Local copy retrieved 2026-01-01,") => Ok(()),
            _ => Err(format!("hosted reply did not state the copy date: {reply}")),
        });
    let stopped = server.finish();
    let fixture = stub.finish().map(|_| ());
    result.and(stopped).and(fixture)
}

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|&word| word.into()).collect()
}

fn checked<'a>(value: &'a Value, status: &str, state: &str) -> Result<&'a Value, String> {
    let hgnc = value
        .get("sources")
        .and_then(Value::as_array)
        .and_then(|sources| {
            sources
                .iter()
                .find(|source| source.get("source") == Some(&json!("hgnc")))
        })
        .ok_or_else(|| format!("missing HGNC outcome: {value}"))?;
    if value.get("status") != Some(&json!(status)) || hgnc.get("state") != Some(&json!(state)) {
        return Err(format!("expected {status} with HGNC {state}: {value}"));
    }
    if !value.to_string().contains("ornithine decarboxylase 1") {
        return Err(format!("stored rows missing: {value}"));
    }
    Ok(hgnc)
}

fn notice(hgnc: &Value) -> Option<&str> {
    hgnc.get("notices")?
        .as_array()?
        .iter()
        .filter_map(|notice| notice.get("message").and_then(Value::as_str))
        .find(|message| message.starts_with("Local copy retrieved"))
}

fn expire(env: &Environment) -> Result<(), String> {
    let path = fs::read_dir(env.root().join("cache/biomcp"))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "sqlite")
        })
        .ok_or("cache missing")?;
    let store = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
    // Updating a bundle row clears its content row, so restore it with a fixed date.
    let revisions: Vec<(String, String)> = store
        .prepare("SELECT name,revision FROM bundle_content")
        .and_then(|mut rows| {
            rows.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect()
        })
        .map_err(|e| e.to_string())?;
    if revisions.is_empty() {
        return Err("seeded bundle has no retrieval date".into());
    }
    store
        .execute("UPDATE bundle SET fresh_until=0", [])
        .map_err(|e| e.to_string())?;
    for (name, revision) in revisions {
        store
            .execute(
                "INSERT INTO bundle_content VALUES(?1,?2,?3)",
                rusqlite::params![name, revision, RETRIEVED],
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
