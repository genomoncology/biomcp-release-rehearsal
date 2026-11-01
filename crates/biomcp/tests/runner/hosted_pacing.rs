//! Feature mcp-hosted: HTTP source headers select independent hosted pacing buckets.
use super::{http::Server, launch::Environment};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
type Check<T = ()> = Result<T, String>;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        super::transcript::trial(super::pool_contract::HOSTED_PACING.0, || {
            check(true).map_err(Into::into)
        }),
        super::transcript::trial("mcp-hosted/pacing/full-rate-keyless", || {
            check(false).map_err(Into::into)
        }),
    ]
}

fn check(keyed: bool) -> Check {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let recordings = recordings(&assets)?;
    let mut server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({})),
        Some(recordings),
        None,
        None,
        &["--hosted".into(), "--rate-share".into(), "0.00001".into()],
    )?;
    let a = keyed.then_some("synthetic-pacing-caller-a");
    let b = keyed.then_some("synthetic-pacing-caller-b");
    let first = call(&server, "PacingA", a)?;
    if first.pointer("/result/isError") != Some(&json!(false)) {
        return Err(format!("first caller did not spend its token: {first}"));
    }
    let spent = call(&server, "PacingSpent", a)?;
    if keyed
        && !spent
            .pointer("/result/structuredContent/sources")
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                sources.iter().any(|source| {
                    source.get("reason").and_then(Value::as_str) == Some("deadline-exceeded")
                })
            })
    {
        return Err(format!(
            "exhausted caller did not reach its typed pacing deadline: {spent}"
        ));
    }
    let other = call(&server, "PacingB", b)?;
    if keyed {
        if other.pointer("/result/isError") != Some(&json!(false)) {
            return Err(format!(
                "another header key shared the exhausted bucket: {other}"
            ));
        }
    } else if spent.pointer("/result/isError") != Some(&json!(false))
        || other.pointer("/result/isError") != Some(&json!(false))
    {
        return Err(format!(
            "keyless calls did not retain their full source rate: {spent}; {other}"
        ));
    }
    server.stop()
}

fn call(server: &Server, query: &str, key: Option<&str>) -> Check<Value> {
    let extra = key.map_or_else(String::new, |key| {
        format!("x-biomcp-key-semantic-scholar-api-key: {key}\r\n")
    });
    super::request_keys::call(
        server,
        &["search", "author", query],
        &format!("{extra}x-biomcp-use: academic\r\n"),
        false,
    )
}

fn recordings(assets: &Environment) -> Check<std::path::PathBuf> {
    let recordings = assets.root().join("replies");
    let source = recordings.join("semantic-scholar");
    std::fs::create_dir_all(&source).map_err(|e| e.to_string())?;
    let original: Value = serde_json::from_str(&super::launch::fixture_text(
        "replies/author/synthetic/semantic-scholar/90d8b5261ee71845c924edbc2407040f440b73da2ab1fd8e51159839ba6ca612.json",
    )?).map_err(|e| e.to_string())?;
    for query in ["PacingA", "PacingSpent", "PacingB"] {
        let mut record = original.clone();
        let old = record
            .get("url")
            .and_then(Value::as_str)
            .ok_or("recorded URL missing")?;
        let (_, rest) = old
            .split_once("&fields=")
            .ok_or("recorded fields missing")?;
        let url = format!(
            "https://api.semanticscholar.org/graph/v1/author/search?query={query}&fields={rest}"
        );
        let hash = Sha256::digest([b"GET\0".as_slice(), url.as_bytes()].concat())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        record
            .as_object_mut()
            .ok_or("recorded request is not an object")?
            .insert("url".into(), json!(url));
        std::fs::write(
            source.join(format!("{hash}.json")),
            serde_json::to_vec(&record).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(recordings)
}
