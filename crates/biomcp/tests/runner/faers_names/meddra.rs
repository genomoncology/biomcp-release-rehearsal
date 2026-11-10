//! An excluded component gates only callers that actually consume it.
use super::{Check, Value, args, invoke, json, query, setup};

pub(super) fn check() -> Check {
    fresh()?;
    continuation()?;
    refusals()
}
fn fresh() -> Check {
    let (env, stub, _) = setup(0, json!({}))?;
    let mut words = args(true);
    words.extend(["--disable-sources".into(), "meddra".into()]);
    let cli = invoke(&env, words.clone(), 0)?;
    let mcp = super::super::mcp::call_with_globals(&env, &words, false, &[])?;
    expanded(&cli, 0)?;
    expanded(&mcp, 0)?;
    if query(&cli)? != query(&mcp)? {
        return Err("MedDRA-off fresh CLI/MCP queries differ".into());
    }
    let requests = stub.finish()?;
    wire(&requests, query(&cli)?, 1)
}
fn continuation() -> Check {
    let (env, stub, no_lookup) = setup(0, json!({}))?;
    let first = invoke(&env, args(true), 0)?;
    expanded(&first, 0)?;
    no_lookup.send(()).map_err(|e| e.to_string())?;
    let mut next = super::super::action(&first, "page")?;
    next.extend([
        "--disable-sources".into(),
        "meddra".into(),
        "--fresh".into(),
    ]);
    let cli = invoke(&env, next.clone(), 0)?;
    let mcp = super::super::mcp::call_with_globals(&env, &next, false, &[])?;
    for value in [&cli, &mcp] {
        expanded(value, 1)?;
        if query(value)? != query(&first)?
            || value.pointer("/query/name_snapshot") != first.pointer("/query/name_snapshot")
        {
            return Err(format!(
                "MedDRA-off continuation lost frozen aliases: {value}"
            ));
        }
    }
    wire(&stub.finish()?, query(&first)?, 3)
}
fn expanded(value: &Value, offset: u32) -> Check {
    if value.get("status") != Some(&json!("complete"))
        || !query(value)?.contains("Brand name")
        || !query(value)?.contains("Typed drug")
        || value.pointer("/query/name_snapshot").is_none()
        || value.pointer("/page/offset") != Some(&json!(offset))
        || !value.to_string().contains("drug-name-search")
    {
        return Err(format!(
            "plain MedDRA-off report silently dropped explicit expansion: {value}"
        ));
    }
    Ok(())
}
fn wire(requests: &[(String, String)], expression: &str, reports: usize) -> Check {
    let mut lookups = 0;
    let mut searches = 0;
    for (_, request) in requests {
        let target = request.split_whitespace().nth(1).ok_or("request target")?;
        lookups += usize::from(target.starts_with("/v1/query?"));
        if target.starts_with("/drug/event.json?") {
            let url = url::Url::parse(&format!("https://synthetic.test{target}"))
                .map_err(|e| e.to_string())?;
            if !url
                .query_pairs()
                .any(|(key, value)| key == "search" && value == expression)
            {
                return Err("MedDRA-off native FAERS expression changed".into());
            }
            searches += 1;
        }
    }
    if lookups != 1 || searches != reports {
        return Err(format!(
            "unexpected MedDRA-off source traffic: lookups={lookups} reports={searches}"
        ));
    }
    Ok(())
}
fn refusals() -> Check {
    let (env, stub, _) = setup(0, json!({}))?;
    let first = invoke(&env, args(true), 0)?;
    let blocked = ["--disable-sources", "meddra"].map(str::to_owned);
    for extra in [
        ["--reaction", "Synthetic reaction"],
        ["--aggregate", "reactions"],
    ] {
        let mut words = args(true);
        words.extend(extra.map(str::to_owned));
        words.extend(blocked.clone());
        let cli = invoke(&env, words.clone(), 4)?;
        let mcp = super::super::mcp::call_with_globals(&env, &words, true, &[])?;
        for value in [cli, mcp] {
            if !value.to_string().contains("disabled")
                || value.get("status") != Some(&json!("failed"))
            {
                return Err(format!("required MedDRA dependency was bypassed: {value}"));
            }
        }
    }
    let mut next = super::super::action(&first, "page")?;
    next.extend(["--disable-sources".into(), "faers".into()]);
    for value in [
        invoke(&env, next.clone(), 4)?,
        super::super::mcp::call_with_globals(&env, &next, true, &[])?,
    ] {
        if !value.to_string().contains("disabled")
            || value.get("result").is_some()
            || value.get("local_error").is_some()
            || value
                .get("next")
                .and_then(Value::as_array)
                .is_some_and(|steps| !steps.is_empty())
        {
            return Err(format!(
                "primary FAERS exclusion reused a saved payload: {value}"
            ));
        }
    }
    wire(&stub.finish()?, query(&first)?, 1)
}
