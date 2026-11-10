use super::{Check, Environment, Value, args, invoke, json, selector, setup};
pub(super) fn complete() -> Check {
    let (env, stub) = setup(0, json!({}))?;
    let literal = invoke(&env, args(false), 0)?;
    if literal.pointer("/page/total") != Some(&json!(1)) {
        return Err(format!("literal changed {literal}"));
    }
    let mut words = args(true);
    words.extend(["--indication".into(), "Synthetic indication".into()]);
    let first = invoke(&env, words.clone(), 0)?;
    let mcp = crate::runner::mcp::call_with_globals(&env, &words, false, &[])?;
    for value in [&first, &mcp] {
        if value.pointer("/page/total") != Some(&json!(2))
            || !selector(value)?.contains("Brand name")
            || !selector(value)?.contains("indications_and_usage:\"Synthetic indication\"")
            || value.pointer("/query/name_snapshot").is_none()
            || !value.to_string().contains("drug-name-search")
        {
            return Err(format!(
                "expanded native union or disclosure missing {value}"
            ));
        }
    }
    typed(&env)?;
    let second = invoke(&env, crate::runner::action(&first, "page")?, 0)?;
    if selector(&first)? != selector(&second)?
        || second.pointer("/page/offset") != Some(&json!(1))
        || first.pointer("/result/items/0/id") == second.pointer("/result/items/0/id")
        || second.pointer("/result/items/0/matched_name").is_some()
        || second
            .get("next")
            .and_then(Value::as_array)
            .is_some_and(|steps| steps.iter().any(|s| s.get("kind") == Some(&json!("page"))))
    {
        return Err(format!(
            "native identity or exhausted page changed {second}"
        ));
    }
    identities(&env, &first, &second)?;
    snapshots(&env, &first)?;
    native(&stub.finish()?, selector(&first)?)?;
    Ok(())
}
fn snapshots(env: &Environment, first: &Value) -> Check {
    let next = crate::runner::action(first, "page")?;
    let index = next
        .iter()
        .position(|s| s == "--name-snapshot")
        .ok_or("snapshot")?;
    let mut missing = next.clone();
    missing.drain(index..=index + 1);
    invoke(env, missing, 2)?;
    let mut bad = next.clone();
    *bad.get_mut(index + 1).ok_or("value")? = "synthetic-missing".into();
    let failed = invoke(env, bad, 3)?;
    crate::runner::action(&failed, "restart")?;
    for disabled in ["mychem", "drugcentral", "openfda-label"] {
        let mut bad = next.clone();
        bad.extend(["--disable-sources".into(), disabled.into()]);
        let failed = invoke(env, bad, 3)?;
        crate::runner::action(&failed, "restart")?;
    }
    let mut changed = next;
    let field = changed
        .iter()
        .position(|word| word == "--indication")
        .ok_or("indication")?;
    *changed.get_mut(field + 1).ok_or("indication value")? = "Changed".into();
    let failed = invoke(env, changed, 3)?;
    crate::runner::action(&failed, "restart")?;
    for input in ["NDA202155", "095a08ac-cf0e-497e-a682-ddef38d6b29c"] {
        let mut native = args(true);
        *native.get_mut(2).ok_or("input")? = input.into();
        invoke(env, native, 2)?;
    }
    let mut bad = args(false);
    bad.extend(["--name-snapshot".into(), "synthetic-missing".into()]);
    invoke(env, bad, 2)?;
    for source in ["mychem", "drugsfda"] {
        let mut bad = args(true);
        let field = bad
            .iter()
            .position(|word| word == "--source")
            .ok_or("source")?;
        *bad.get_mut(field + 1).ok_or("source value")? = source.into();
        invoke(env, bad, 2)?;
    }
    Ok(())
}
pub(super) fn controls() -> Check {
    for mode in [1, 2, 3, 4, 5, 6] {
        let (env, stub) = setup(mode, json!({}))?;
        let value = invoke(&env, args(true), if mode == 6 { 4 } else { 0 })?;
        if mode == 6 && value.get("status") != Some(&json!("failed")) {
            return Err("lookup hid primary failure".into());
        }
        if mode != 6 {
            let expression = selector(&value)?;
            if expression.len() > 4096
                || mode == 4 && expression.contains("Long alias")
                || [1, 2, 3].contains(&mode) && expression.contains("Brand name")
                || [1, 3].contains(&mode) && value.get("status") != Some(&json!("partial"))
                || mode == 5 && !expression.contains(r#"Brand \"quoted\" \\name"#)
            {
                return Err(format!("uncertainty or bound dishonest {value}"));
            }
        }
        stub.finish()?;
    }
    let (env, stub) = setup(0, json!({"drug_search_name_limit":1}))?;
    let value = invoke(&env, args(true), 0)?;
    if selector(&value)?.contains("Brand name") {
        return Err("configured name cap ignored".into());
    }
    stub.finish()?;
    let (env, stub) = setup(0, json!({"disable_sources":["mychem","chembl"]}))?;
    let value = invoke(&env, args(true), 0)?;
    if selector(&value)?.contains("Brand name") {
        return Err("disabled discovery supplied names".into());
    }
    stub.finish()?;
    Ok(())
}
pub(super) fn tier() -> Check {
    let (env, stub) = setup(7, json!({}))?;
    let first = invoke(&env, args(true), 0)?;
    if selector(&first)?.contains(".exact") {
        return Err("empty exact tier did not advance".into());
    }
    let mut next = crate::runner::action(&first, "page")?;
    next.push("--fresh".into());
    let second = invoke(&env, next, 0)?;
    if selector(&first)? != selector(&second)? {
        return Err("selected native tier changed".into());
    }
    let requests = stub.finish()?;
    if requests
        .iter()
        .filter(|(_, s)| s.contains("/v1/query?"))
        .count()
        != 1
    {
        return Err("continuation re-resolved names".into());
    }
    Ok(())
}

fn typed(env: &Environment) -> Check {
    let path = env.root().join("config/biomcp/config.json");
    let mut config: Value =
        serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    config
        .as_object_mut()
        .ok_or("fixture config object")?
        .insert("mcp_tool_mode".into(), json!("typed"));
    env.config(&config).map_err(|e| e.to_string())?;
    let replies = crate::runner::mcp::exchange_with_globals(
        env,
        &[crate::runner::mcp::request(
            json!(1),
            "tools/call",
            json!({
                "name":"drug", "arguments":{"action":"search", "search":{
                    "query":"Typed drug", "source":"openfda-label", "limit":1,
                    "expand_drug_names":true, "indication":"Synthetic indication"
                }}
            }),
        )],
        &[],
    )?;
    let value = replies
        .first()
        .and_then(|value| value.pointer("/result/structuredContent"))
        .ok_or_else(|| format!("no typed label result: {replies:?}"))?;
    if value.pointer("/page/total") != Some(&json!(2)) || !selector(value)?.contains("Brand name") {
        return Err(format!("typed expansion differs: {value}"));
    }
    Ok(())
}

fn identities(env: &Environment, first: &Value, second: &Value) -> Check {
    for (value, id, version, date) in [
        (
            first,
            "095a08ac-cf0e-497e-a682-ddef38d6b29c",
            "7",
            "2021-06-15",
        ),
        (
            second,
            "a454cd24-0c6d-46e8-b1e4-197388606175",
            "38",
            "2026-04-24",
        ),
    ] {
        if value.pointer("/result/items/0/id") != Some(&json!(id))
            || value.pointer("/result/items/0/version") != Some(&json!(version))
        {
            return Err(format!("native SPL identity or version changed: {value}"));
        }
        let get = crate::runner::action(value, "disambiguate")?;
        if get.get(2).map(String::as_str) != Some(id)
            || get.iter().any(|s| s == "--expand-drug-names")
        {
            return Err(format!("SPL read was expanded: {get:?}"));
        }
        let card = invoke(env, get, 0)?;
        if card.pointer("/result/updated") != Some(&json!(date))
            || card.pointer("/result/header/version") != Some(&json!(version))
        {
            return Err(format!("SPL revisions were conflated: {card}"));
        }
    }
    Ok(())
}
fn native(requests: &[(String, String)], expected: &str) -> Check {
    let mut union_pages = 0;
    for (_, request) in requests {
        let target = request.split_whitespace().nth(1).ok_or("native request")?;
        if !target.starts_with("/drug/label.json?") {
            continue;
        }
        let url = url::Url::parse(&format!("https://synthetic.test{target}"))
            .map_err(|e| e.to_string())?;
        let selector = url
            .query_pairs()
            .find(|(key, _)| key == "search")
            .ok_or("native selector")?
            .1;
        if selector.starts_with("((") {
            union_pages += 1;
            if selector != expected {
                return Err(format!(
                    "native alias union or literal indication changed: {selector}"
                ));
            }
        }
    }
    if union_pages != 2 {
        return Err(format!(
            "expected initial and following native union pages; got {union_pages}"
        ));
    }
    Ok(())
}
