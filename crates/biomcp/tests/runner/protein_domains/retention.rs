//! A refused fragment leaves native entry membership intact but disqualifies implicit reuse.
use super::{Environment, Stub, Value, call, json, setup};

pub(super) fn healthy(env: &Environment, first: &Value) -> Result<(), String> {
    let repeated = call(env, &["protein", "domains", "P04637"], 0)?;
    if first.pointer("/status") != Some(&json!("complete"))
        || repeated.pointer("/status") != first.pointer("/status")
        || pool(first)? != pool(&repeated)?
        || first.pointer("/result/items") != repeated.pointer("/result/items")
    {
        return Err(format!("healthy domain window was rebuilt: {repeated}"));
    }
    Ok(())
}

pub(super) fn refused() -> Result<(), String> {
    let stub = Stub::for_source("www.ebi.ac.uk", "application/json", |_| {
        Ok(json!({"count":2,"next":null,"previous":null,"results":[
            entry("IPR002117", true), entry("IPR010991", false)
        ]})
        .to_string())
    })?;
    let env = setup(&stub)?;
    let args = ["protein", "domains", "P04637", "--limit", "1"];
    let first = call(&env, &args, 0)?;
    if first.pointer("/status") != Some(&json!("partial"))
        || first.pointer("/result/items/0/id") != Some(&json!("IPR002117"))
        || first.pointer("/page/total") != Some(&json!(2))
        || first.pointer("/result/native_total") != Some(&json!(2))
        || first.pointer("/sources/0/state") != Some(&json!("ok"))
        || first
            .pointer("/result/items/0/locations/0/fragments")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(1)
        || first
            .get("problems")
            .and_then(Value::as_array)
            .is_none_or(|problems| {
                problems
                    .iter()
                    .all(|p| p.get("field") != Some(&json!("fragments.end")))
            })
    {
        return Err(format!(
            "fragment refusal lost admitted entries or source-Ok guidance: {first}"
        ));
    }
    let next = super::super::action(&first, "page")?;
    let words: Vec<_> = next.iter().map(String::as_str).collect();
    let second = call(&env, &words, 0)?;
    if pool(&first)? != pool(&second)?
        || second.pointer("/status") != Some(&json!("partial"))
        || second.pointer("/result/items/0/id") != Some(&json!("IPR010991"))
    {
        return Err(format!("explicit domain continuation changed: {second}"));
    }
    let repeated = call(&env, &args, 0)?;
    if pool(&first)? == pool(&repeated)? {
        return Err(format!(
            "incomplete domain pool became the implicit default: {repeated}"
        ));
    }
    if first.pointer("/result/items") != repeated.pointer("/result/items")
        || first.pointer("/status") != repeated.pointer("/status")
    {
        return Err(format!(
            "implicit domain reconstruction changed records: {repeated}"
        ));
    }
    let kept = call(&env, &words, 0)?;
    if pool(&second)? != pool(&kept)?
        || second.pointer("/result/items") != kept.pointer("/result/items")
        || second.pointer("/status") != kept.pointer("/status")
    {
        return Err(format!("old explicit domain page was lost: {kept}"));
    }
    stub.finish()?;
    Ok(())
}

fn pool(value: &Value) -> Result<&str, String> {
    value
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("domain window lost pool identity: {value}"))
}

fn entry(id: &str, refused: bool) -> Value {
    let mut fragments = vec![json!({"start":10,"end":20,"dc-status":"CONTINUOUS"})];
    if refused {
        fragments.push(json!({"start":30,"end":25,"dc-status":"CONTINUOUS"}));
    }
    json!({"metadata":{"accession":id,"name":"Synthetic domain","source_database":"interpro",
        "type":"domain","member_databases":{}},"proteins":[{"accession":"p04637","protein_length":393,
        "organism":"9606","entry_protein_locations":[{"fragments":fragments}]}]})
}
