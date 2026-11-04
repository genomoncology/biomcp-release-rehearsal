//! Features adverse-event-get and maude-reports: fetched exact records need no retained token.
use super::super::{faers_get, launch::Environment, maude, network_stub::Stub, schema::Registry};
use serde_json::{Value, json};
use std::{fs, sync::mpsc};

pub(super) fn check(device: bool, section: Option<&str>) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let path = env.root().join(format!(
        "cache/biomcp/cache-{}-{}.sqlite",
        biomcp_fetch::CACHE_NAMESPACE_VERSION,
        biomcp_query::store::CACHE_NAMESPACE_VERSION
    ));
    let (writer, held) = mpsc::channel();
    let body = if device {
        maude::response(maude::row())
    } else {
        faers_get::native().to_string()
    };
    let stub = Stub::for_source("api.fda.gov", "application/json", move |request| {
        if !request.contains(if device {
            "/device/event.json?"
        } else {
            "/drug/event.json?"
        }) {
            return Err(format!("unexpected exact-report request: {request}"));
        }
        let db = rusqlite::Connection::open(&path).map_err(|e| e.to_string())?;
        db.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| e.to_string())?;
        writer.send(db).map_err(|e| e.to_string())?;
        Ok(body.clone())
    })?;
    stub.retain(env.shared_root());
    let ca = env.root().join("source-ca.pem");
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    let mut args = if device {
        vec!["get", "adverse-event", "160", "--source", "maude"]
    } else {
        vec!["get", "adverse-event", "100"]
    }
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    if let Some(section) = section {
        args.push(section.to_owned());
        if matches!(
            section,
            "unii"
                | "rxcui"
                | "product-ndc"
                | "package-ndc"
                | "spl-id"
                | "spl-set-id"
                | "application-number"
        ) {
            args.extend(["--drug-index".into(), "0".into()]);
        }
    }
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    drop(held.try_recv().map_err(|e| e.to_string())?);
    let requests = stub.finish()?;
    let answer: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    Registry::embedded()?.validate(&answer, &args, true)?;
    let id = if device { "160" } else { "100" };
    if !output.status.success() || requests.len() != 1 || !retained_answer(&answer, id) {
        return Err(format!(
            "failed publication erased exact report or returned a false snapshot: {answer}"
        ));
    }
    Ok(())
}

fn retained_answer(answer: &Value, id: &str) -> bool {
    answer.pointer("/result/header/id") == Some(&json!(id))
        && answer.pointer("/result/snapshot").is_none()
        && answer
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
        && answer
            .get("next")
            .and_then(Value::as_array)
            .is_some_and(|next| {
                next.len() == 1 && next.iter().all(|n| n.get("kind") == Some(&json!("fresh")))
            })
}
