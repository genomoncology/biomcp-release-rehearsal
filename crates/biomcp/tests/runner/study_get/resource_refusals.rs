//! Feature study-get: refused resource records retain their fact across stored reads.
use super::{rows, seed};
use crate::runner::{configuration::invoke, launch::Environment};
use serde_json::{Value, json};
use std::path::Path;

pub(super) fn run() -> Result<(), String> {
    for kind in ["refused", "optional", "filtered"] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let mut catalog = rows();
        let mut resource = json!({"resourceId":"synthetic_resource","displayName":"Synthetic resource",
            "description":"Synthetic description","resourceType":"STUDY","priority":"1",
            "openByDefault":false,"cancerStudyIdentifier":"synthetic_0","sampleCount":2,"patientCount":2});
        if kind == "optional" {
            *resource.get_mut("priority").ok_or("priority")? = json!(42);
        }
        let mut resources = vec![resource.clone()];
        if kind != "optional" {
            *resource.get_mut("resourceId").ok_or("resource id")? = json!(42);
            resources.insert(0, resource);
        }
        *catalog
            .pointer_mut("/0/resourceCounts")
            .ok_or("resources")? = json!(resources);
        let replay = seed(&env, catalog, 200)?;
        call(&env, &replay, &["sync", "cbioportal-studies"])?;
        let id = if kind == "filtered" {
            "synthetic_1"
        } else {
            "synthetic_0"
        };
        let value = call(&env, &replay, &["get", "study", id])?;
        let expected = if kind == "refused" {
            "partial"
        } else {
            "complete"
        };
        if value.pointer("/status") != Some(&json!(expected)) {
            return Err(format!("{kind} resource changed item status: {value}"));
        }
        if kind != "filtered"
            && value
                .pointer("/result/study/resourceCounts")
                .and_then(Value::as_array)
                .map(Vec::len)
                != Some(1)
        {
            return Err(format!(
                "{kind} resource lost its accepted sibling: {value}"
            ));
        }
        if value.to_string().contains("refusals") || value.to_string().contains("dropped") {
            return Err("private resource fact entered public serialization".into());
        }
        if kind == "refused" {
            retained(&env, &replay)?;
        } else if kind == "filtered" {
            let page = call(&env, &replay, &["search", "study", "Synthetic Atlas 1"])?;
            if page.pointer("/status") != Some(&json!("complete")) {
                return Err(format!("unselected refusal changed search status: {page}"));
            }
        }
    }
    Ok(())
}
fn retained(env: &Environment, replay: &Path) -> Result<(), String> {
    let first = call(env, replay, &["search", "study", "atlas"])?;
    let id = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("pool")?;
    if first.pointer("/status") != Some(&json!("partial")) {
        return Err(format!("resource refusal lost from initial page: {first}"));
    }
    seed(env, rows(), 200)?;
    call(env, replay, &["sync", "cbioportal-studies"])?;
    let old = call(
        env,
        replay,
        &["search", "study", "atlas", "--pool", id, "--offset", "5"],
    )?;
    if old.pointer("/status") != Some(&json!("partial"))
        || old.pointer("/result/snapshot/revision") != first.pointer("/result/snapshot/revision")
    {
        return Err(format!("catalog refresh changed retained refusal: {old}"));
    }
    Ok(())
}
fn call(env: &Environment, replay: &Path, words: &[&str]) -> Result<Value, String> {
    let mut args = env.globals(Some(replay), false);
    args.extend(words.iter().map(|s| (*s).to_owned()));
    invoke(env, args, 0)
}
