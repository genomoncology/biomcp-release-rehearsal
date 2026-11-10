//! Synthetic HTTP 200 membership isolates refusals from source failure and field problems.
use super::{Environment, Value, json};
use crate::runner::{configuration::invoke, network_stub::Stub};
use std::fs;

pub(super) fn check(refused: bool) -> Result<(), String> {
    let (env, stub) = setup(refused)?;
    let args = ["pathway", "events", "R-HSA-123", "--limit", "1"]
        .map(str::to_owned)
        .to_vec();
    let first = invoke(&env, args.clone(), 0)?;
    let status = if refused { "partial" } else { "complete" };
    if first.pointer("/status") != Some(&json!(status))
        || first.pointer("/result/items/0/id") != Some(&json!("R-HSA-124"))
        || first.pointer("/result/page/total") != Some(&json!(2))
        || !first
            .get("sources")
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                !sources.is_empty() && sources.iter().all(|s| s.get("state") == Some(&json!("ok")))
            })
    {
        return Err(format!(
            "synthetic event membership/source state changed: {first}"
        ));
    }
    if refused
        && !first
            .get("problems")
            .and_then(Value::as_array)
            .is_some_and(|problems| {
                problems
                    .iter()
                    .any(|p| p.get("field") == Some(&json!("events.id")))
            })
    {
        return Err(format!("invalid event identity was not diagnosed: {first}"));
    }
    let next = crate::runner::action(&first, "page")?;
    let second = invoke(&env, next.clone(), 0)?;
    if pool(&first)? != pool(&second)?
        || second.pointer("/result/items/0/id") != Some(&json!("R-HSA-125"))
        || second.pointer("/status") != Some(&json!(status))
    {
        return Err(format!("explicit events continuation changed: {second}"));
    }
    let repeated = invoke(&env, args, 0)?;
    if first.pointer("/result/items") != repeated.pointer("/result/items")
        || repeated.pointer("/status") != Some(&json!(status))
    {
        return Err(format!("repeated event records changed: {repeated}"));
    }
    if refused {
        if pool(&first)? == pool(&repeated)? {
            return Err(format!(
                "incomplete pathway pool became the implicit default: {repeated}"
            ));
        }
    } else if pool(&first)? != pool(&repeated)? {
        return Err(format!("healthy pathway window was rebuilt: {repeated}"));
    }
    // The old explicit observation remains readable even after another implicit publication.
    let kept = invoke(&env, next, 0)?;
    if pool(&second)? != pool(&kept)?
        || second.pointer("/result/items") != kept.pointer("/result/items")
        || second.pointer("/status") != kept.pointer("/status")
    {
        return Err(format!("old explicit event page was lost: {kept}"));
    }
    stub.finish()?;
    Ok(())
}

fn pool(value: &Value) -> Result<&str, String> {
    value
        .pointer("/result/page/pool/id")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("event window lost pool identity: {value}"))
}

fn setup(refused: bool) -> Result<(Environment, Stub), String> {
    let stub = Stub::for_sources(
        &["reactome.org"],
        |_| "application/json",
        move |request| {
            let body = if request.contains("/data/query/R-HSA-123") {
                json!({"stId":"R-HSA-123","stIdVersion":"R-HSA-123.1","displayName":"Synthetic pathway",
                    "speciesName":"Homo sapiens","schemaClass":"Pathway"})
            } else if request.contains("/data/pathway/R-HSA-123/containedEvents") {
                let mut rows = vec![event(124, "R-HSA-124"), event(125, "R-HSA-125")];
                if refused {
                    rows.push(event(126, "invalid"));
                }
                json!(rows)
            } else if request.contains("/data/database/version") {
                json!(98)
            } else {
                return Err(format!("unexpected Reactome request: {request}"));
            };
            Ok(body.to_string())
        },
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("source-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok((env, stub))
}

fn event(db: u32, id: &str) -> Value {
    json!({"dbId":db,"stId":id,"displayName":"Synthetic event",
        "speciesName":"Homo sapiens","schemaClass":"Pathway"})
}
