//! Feature mcp-hosted: old labels restore and use filtering never changes another caller's evidence.
use super::{ACADEMIC, KEY, call};
use crate::runner::{http::Server, launch::Environment};
use serde_json::json;

pub(super) fn check() -> Result<(), String> {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let replay = recordings(&assets)?;
    let mut server = Server::configured_globals(
        "127.0.0.1:0",
        None,
        Some(replay),
        None,
        None,
        &["--hosted".into()],
    )?;
    let words = [
        "article",
        "citation-evidence",
        "33123537",
        "17611497",
        "--fulltext",
    ];
    let first = call(&server, &words, ACADEMIC, false)?;
    let original = first
        .pointer("/result/structuredContent/result/fulltext")
        .ok_or("missing full text")?
        .clone();
    if original.get("state") != Some(&json!("data")) {
        return Err(format!("academic evidence missing: {first}"));
    }
    let database = server.environment().root().join("case-cache/replay.sqlite");
    let db = rusqlite::Connection::open(database).map_err(|e| e.to_string())?;
    let (id, outcomes, payload): (String, String, String) = db
        .query_row(
            "SELECT id,outcomes,payload FROM citation_evidence",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;
    // Model exactly the old label bytes; source identities, ownership and content stay intact.
    let old = |text: &str| {
        text.replace("\"use\":\"per-record\"", "\"use\":\"restricted\"")
            .replace("\"use\":\"open\"", "\"use\":\"restricted\"")
    };
    db.execute(
        "UPDATE citation_evidence SET outcomes=?1,payload=?2 WHERE id=?3",
        rusqlite::params![old(&outcomes), old(&payload), id],
    )
    .map_err(|e| e.to_string())?;
    drop(db);
    let args: Vec<_> = words.into_iter().chain(["--pool", id.as_str()]).collect();
    let restored = call(&server, &args, ACADEMIC, false)?;
    if restored.pointer("/result/structuredContent/result/fulltext") != Some(&original) {
        return Err(format!("pre-change continuation failed: {restored}"));
    }
    let commercial =
        format!("x-biomcp-use: commercial\r\nx-biomcp-key-semantic-scholar-api-key: {KEY}\r\n");
    let filtered = call(&server, &args, &commercial, false)?;
    if filtered.pointer("/result/structuredContent/result/fulltext/reason")
        != Some(&json!("disabled"))
        || filtered
            .pointer("/result/structuredContent/result/fulltext/data")
            .is_some()
    {
        return Err(format!(
            "commercial continuation leaked full text: {filtered}"
        ));
    }
    let academic = call(&server, &args, ACADEMIC, false)?;
    if academic.pointer("/result/structuredContent/result/fulltext") != Some(&original) {
        return Err(format!(
            "commercial filtering changed academic state: {academic}"
        ));
    }
    server.stop()
}

fn recordings(assets: &Environment) -> Result<std::path::PathBuf, String> {
    let replay = assets.root().join("replies");
    biomcp_test_support::capture::copy_capture(
        "article/evidence/source",
        &replay,
        |source, record| {
            if source == "pmc-oa" {
                for exchange in record["exchanges"].as_array_mut().into_iter().flatten() {
                    let Some(body) = exchange.pointer_mut("/body/value") else {
                        continue;
                    };
                    if let Some(text) = body.as_str() {
                        *body = json!(text.replace(
                            "\"license_code\": \"CC BY\"",
                            "\"license_code\": \"CC BY NC\""
                        ));
                    } else if let Some(license) = body.get_mut("license_code") {
                        *license = json!("CC BY NC");
                    }
                }
            }
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(replay)
}
