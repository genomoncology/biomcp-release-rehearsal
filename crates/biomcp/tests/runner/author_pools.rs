//! Public searches distinguish a reusable native window from refused source membership.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::fs;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    [
        (
            "author-search/healthy-window-reuses-pool",
            healthy as fn() -> Result<(), String>,
        ),
        ("author-search/refused-window-is-explicit-only", refused),
        ("author-search/refused-extension-retires-default", extension),
    ]
    .into_iter()
    .map(|(name, check)| super::transcript::trial(name, move || check().map_err(Into::into)))
    .collect()
}

fn setup(mixed_first: bool) -> Result<(Environment, Stub), String> {
    let stub = Stub::for_sources(
        &["api.semanticscholar.org"],
        |_| "application/json",
        move |request| {
            if !request.contains("/author/search?") {
                return Err(format!("unexpected author request: {request}"));
            }
            let later = request.contains("offset=2");
            let offset = if later { 2 } else { 0 };
            let id = if later { "903" } else { "901" };
            let other = if later || mixed_first {
                "invalid"
            } else {
                "902"
            };
            Ok(json!({"offset":offset,"next":offset+2,"total":6,"data":[
                {"authorId":id,"name":"Synthetic first author"},
                {"authorId":other,"name":"Synthetic second author"}
            ]})
            .to_string())
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

fn search(env: &Environment, pool: Option<&str>, offset: &str, exit: i32) -> Result<Value, String> {
    let mut args = vec![
        "search",
        "author",
        "Synthetic pool author",
        "--source",
        "semantic-scholar",
        "--limit",
        "2",
        "--offset",
        offset,
    ];
    if let Some(pool) = pool {
        args.extend(["--pool", pool]);
    }
    invoke(env, args.into_iter().map(str::to_owned).collect(), exit)
}

fn pool(value: &Value) -> Result<&str, String> {
    value
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("author window lost pool identity: {value}"))
}

fn same(first: &Value, repeated: &Value) -> Result<(), String> {
    if pool(first)? != pool(repeated)?
        || first.pointer("/result/items") != repeated.pointer("/result/items")
        || first.pointer("/status") != repeated.pointer("/status")
    {
        return Err(format!(
            "retained author window changed: {first}; {repeated}"
        ));
    }
    Ok(())
}

fn partial(value: &Value, id: &str) -> Result<(), String> {
    if value.pointer("/status") != Some(&json!("partial"))
        || value.pointer("/result/items/0/id") != Some(&json!(id))
        || value
            .pointer("/result/items")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(1)
        || value
            .get("problems")
            .and_then(Value::as_array)
            .is_none_or(|problems| {
                problems.iter().all(|problem| {
                    problem.get("field") != Some(&json!("author_id"))
                        || problem.get("source") != Some(&json!("semantic-scholar"))
                })
            })
        || value
            .get("sources")
            .and_then(Value::as_array)
            .is_none_or(|sources| {
                sources.is_empty()
                    || sources
                        .iter()
                        .any(|source| source.get("state") != Some(&json!("ok")))
            })
    {
        return Err(format!(
            "refused native identity lost its partial page: {value}"
        ));
    }
    Ok(())
}

fn healthy() -> Result<(), String> {
    let (env, stub) = setup(false)?;
    let first = search(&env, None, "0", 0)?;
    if first.pointer("/status") != Some(&json!("complete"))
        || first.pointer("/result/items/0/id") != Some(&json!("semantic-scholar:901"))
        || first.pointer("/result/items/1/id") != Some(&json!("semantic-scholar:902"))
    {
        return Err(format!("healthy native author window changed: {first}"));
    }
    // A native next page exists but is deliberately unread: the observed window is still reusable.
    super::action(&first, "page")?;
    same(&first, &search(&env, None, "0", 0)?)?;
    if stub.finish()?.len() != 1 {
        return Err("healthy repeat fetched unread native pages".into());
    }
    Ok(())
}

fn refused() -> Result<(), String> {
    let (env, stub) = setup(true)?;
    let first = search(&env, None, "0", 0)?;
    partial(&first, "semantic-scholar:901")?;
    same(&first, &search(&env, Some(pool(&first)?), "0", 0)?)?;
    let repeated = search(&env, None, "0", 0)?;
    partial(&repeated, "semantic-scholar:901")?;
    if pool(&first)? == pool(&repeated)? {
        return Err(format!(
            "incomplete author pool became the implicit default: {repeated}"
        ));
    }
    // Reconstruction may legitimately read the same HTTP cache; identity proves it is a new pool.
    same(&first, &search(&env, Some(pool(&first)?), "0", 0)?)?;
    stub.finish()?;
    Ok(())
}

fn extension() -> Result<(), String> {
    let (env, stub) = setup(false)?;
    let first = search(&env, None, "0", 0)?;
    let next = super::action(&first, "page")?;
    let extended = invoke(&env, next, 0)?;
    partial(&extended, "semantic-scholar:903")?;
    if pool(&first)? != pool(&extended)? {
        return Err(format!(
            "native extension replaced its explicit observation: {extended}"
        ));
    }
    same(&extended, &search(&env, Some(pool(&extended)?), "2", 0)?)?;
    same(&first, &search(&env, Some(pool(&first)?), "0", 0)?)?;
    let rebuilt = search(&env, None, "0", 0)?;
    if pool(&first)? == pool(&rebuilt)?
        || first.pointer("/result/items") != rebuilt.pointer("/result/items")
    {
        return Err(format!(
            "refused extension remained the implicit default: {rebuilt}"
        ));
    }
    same(&rebuilt, &search(&env, None, "0", 0)?)?;
    same(&extended, &search(&env, Some(pool(&extended)?), "2", 0)?)?;
    if stub.finish()?.len() != 2 {
        return Err("author windows unexpectedly fetched beyond the emitted continuation".into());
    }
    Ok(())
}

tool_only! {}
