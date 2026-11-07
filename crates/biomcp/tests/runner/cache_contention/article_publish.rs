//! Feature article-search: a writer arriving during fetch cannot erase checked items.
use super::super::{launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::{fs, sync::mpsc};

pub(super) fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let path = env.root().join(format!(
        "cache/biomcp/cache-{}-{}.sqlite",
        biomcp_fetch::CACHE_NAMESPACE_VERSION,
        biomcp_query::store::CACHE_NAMESPACE_VERSION
    ));
    let (writer, held) = mpsc::channel();
    let stub = Stub::for_source(
        "api.semanticscholar.org",
        "application/json",
        move |request| {
            if !request.contains("/graph/v1/paper/search") {
                return Err(format!("unexpected article request: {request}"));
            }
            let db = rusqlite::Connection::open(&path).map_err(|e| e.to_string())?;
            db.execute_batch("BEGIN IMMEDIATE")
                .map_err(|e| e.to_string())?;
            writer.send(db).map_err(|e| e.to_string())?;
            Ok(json!({"total":2,"offset":0,"data":[{
                "paperId":"0123456789abcdef0123456789abcdef01234567",
                "title":"Synthetic fetched article","externalIds":{},"year":2026
            }]})
            .to_string())
        },
    )?;
    stub.retain(env.shared_root());
    let ca = env.root().join("source-ca.pem");
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    let args = [
        "search",
        "article",
        "synthetic fetched",
        "--source",
        "semantic-scholar",
        "--limit",
        "1",
        "--exact-term",
    ]
    .map(str::to_owned);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    drop(held.try_recv().map_err(|e| e.to_string())?);
    let requests = stub.finish()?;
    let answer: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if requests.len() != 1
        || answer.pointer("/result/items/0/title").is_none()
            && answer.pointer("/result/items/0/header/title").is_none()
        || answer.pointer("/page/pool").is_some_and(|p| !p.is_null())
        || !answer
            .get("sources")
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                sources.iter().any(|s| {
                    s.get("notices").and_then(Value::as_array).is_some_and(|n| {
                        n.iter()
                            .any(|n| n.get("kind") == Some(&json!("cache-problem")))
                    })
                })
            })
        || !answer
            .get("next")
            .and_then(Value::as_array)
            .is_some_and(|next| {
                next.iter().any(|n| {
                    n.get("kind") == Some(&json!("fresh")) || n.get("kind") == Some(&json!("retry"))
                })
            })
    {
        return Err(format!(
            "late store contention erased fetched article or continuation safety: exit={:?}; reply={answer}",
            output.status.code()
        ));
    }
    Ok(())
}
