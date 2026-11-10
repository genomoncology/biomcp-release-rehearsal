//! Explicit FAERS aliases retain native counts and frozen, caller-authorized membership.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::sync::mpsc;
type Check = Result<(), String>;
mod meddra;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    [
        (
            crate::runner::required_contract::FAERS_NAMES.0,
            complete as fn() -> Check,
        ),
        (
            "adverse-event-search/alias-resolution-uncertainty",
            uncertainty,
        ),
        (
            "adverse-event-search/alias-caller-controls-and-bounds",
            controls,
        ),
    ]
    .into_iter()
    .map(|(name, check)| super::transcript::trial(name, move || check().map_err(Into::into)))
    .collect()
}

fn setup(mode: u8, extra: Value) -> Result<(Environment, Stub, mpsc::Sender<()>), String> {
    let (no_lookup, stopped) = mpsc::channel();
    let stub = Stub::for_sources(
        &["mychem.info", "api.fda.gov"],
        |_| "application/json",
        move |request| {
            if request.contains("/v1/query?") {
                if stopped.try_recv().is_ok() {
                    return Err("a frozen continuation repeated name resolution".into());
                }
                return Ok(lookup(mode).to_string());
            }
            if !request.contains("/drug/event.json?search=") {
                return Err(format!("unexpected FAERS request: {request}"));
            }
            if mode == 6 {
                return Ok(json!({"meta":"malformed"}).to_string());
            }
            if request.contains("&count=") {
                return Ok(json!({"meta":{"last_updated":"2024-09-01"},"results":[{"term":"SYNTHETIC","count":2}]}).to_string());
            }
            let offset = u32::from(request.contains("skip=1"));
            Ok(json!({"meta":{"last_updated":"2024-09-01","results":{"skip":offset,"limit":1,"total":2}},
            "results":[{"safetyreportid":(100+offset).to_string(),"safetyreportversion":"1","receivedate":"20240102",
                "serious":"1","patient":{"drug":[],"reaction":[]}}]}).to_string())
        },
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    stub.retain(env.shared_root());
    let ca = env.root().join("source-ca.pem");
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    let mut config = json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,
        "disable_sources":["chembl","openfda-label"]});
    config
        .as_object_mut()
        .ok_or("config")?
        .extend(extra.as_object().ok_or("extra config")?.clone());
    env.config(&config).map_err(|e| e.to_string())?;
    Ok((env, stub, no_lookup))
}

fn lookup(mode: u8) -> Value {
    if mode == 3 {
        return json!({"total":"malformed","hits":[]});
    }
    let alias = match mode {
        4 => "Long alias ".repeat(150),
        5 => "Brand \"quoted\" \\name".into(),
        _ => "Brand name".into(),
    };
    let row = |id: &str| json!({"_id":id,"drugcentral":{"synonyms":["Typed drug",alias]}});
    let hits = if mode == 2 {
        vec![
            row("KTUFNOKKBVMGRW-UHFFFAOYSA-N"),
            row("YLMAHDNUQAMNNX-UHFFFAOYSA-N"),
        ]
    } else {
        vec![row("KTUFNOKKBVMGRW-UHFFFAOYSA-N")]
    };
    json!({"total":if mode == 1 { 2 } else { hits.len() },"hits":hits})
}

fn args(expand: bool) -> Vec<String> {
    let mut words = ["search", "adverse-event", "Typed drug", "--limit", "1"]
        .map(str::to_owned)
        .to_vec();
    if expand {
        words.push("--expand-drug-names".into());
    }
    words
}

fn query(value: &Value) -> Result<&str, String> {
    value
        .pointer("/result/source_query")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing source query: {value}"))
}

fn complete() -> Check {
    let (env, stub, no_lookup) = setup(0, json!({}))?;
    let literal = invoke(&env, args(false), 0)?;
    if query(&literal)?.contains("Brand name") {
        return Err("literal default expanded names".into());
    }
    let mut words = args(true);
    words.extend(
        [
            "--reaction",
            "Synthetic reaction",
            "--serious",
            "death",
            "--received-from",
            "2024-01",
            "--received-to",
            "2024-12",
        ]
        .map(str::to_owned),
    );
    let first = invoke(&env, words.clone(), 0)?;
    check_expanded(&first)?;
    let mcp = super::mcp::call_with_globals(&env, &words, false, &[])?;
    if query(&mcp)? != query(&first)? || mcp.get("status") != first.get("status") {
        return Err("MCP expansion differs".into());
    }
    let mut count = words;
    count.extend(["--aggregate".into(), "reactions".into()]);
    let counted = invoke(&env, count, 0)?;
    if query(&counted)? != query(&first)?
        || counted.pointer("/result/reaction_counts/items/0/count") != Some(&json!(2))
    {
        return Err(format!(
            "aggregate did not use the same native OR: {counted}"
        ));
    }
    no_lookup.send(()).map_err(|e| e.to_string())?;
    let mut next = super::action(&first, "page")?;
    next.push("--fresh".into());
    let second = invoke(&env, next.clone(), 0)?;
    let mcp_next = super::mcp::call_with_globals(&env, &next, false, &[])?;
    if query(&mcp_next)? != query(&second)? {
        return Err("MCP continuation re-resolved names".into());
    }
    if query(&second)? != query(&first)? || second.pointer("/page/offset") != Some(&json!(1)) {
        return Err(format!(
            "continuation changed resolved membership: {second}"
        ));
    }
    let requests = stub.finish()?;
    verify_native(&requests, query(&first)?)?;
    if requests
        .iter()
        .filter(|(_, line)| line.contains("/v1/query?"))
        .count()
        != 1
    {
        return Err("warm resolution was not retained".into());
    }
    Ok(())
}

fn check_expanded(value: &Value) -> Check {
    let expression = query(value)?;
    if value.get("status") != Some(&json!("complete"))
        || value.pointer("/page/total") != Some(&json!(2))
        || !expression.contains("Brand name")
        || !expression.contains("Typed drug")
        || !expression.contains("seriousnessdeath:1")
        || !expression.contains("receivedate:[20240101 TO 20241231]")
        || !value.to_string().contains("drug-name-search")
        || value.pointer("/query/name_snapshot").is_none()
    {
        return Err(format!(
            "expanded query lost native facts or disclosure: {value}"
        ));
    }
    Ok(())
}

fn uncertainty() -> Check {
    let (env, stub, _) = setup(6, json!({}))?;
    let failed = invoke(&env, args(true), 4)?;
    if failed.get("status") != Some(&json!("failed")) {
        return Err("successful name lookup relabelled failed FAERS as an answer".into());
    }
    stub.finish()?;
    for mode in [1, 2, 3] {
        let (env, stub, _) = setup(mode, json!({}))?;
        let reply = invoke(&env, args(true), 0)?;
        if query(&reply)?.contains("Brand name")
            || !reply.to_string().contains("drug-name-search")
            || mode != 2 && reply.get("status") != Some(&json!("partial"))
        {
            return Err(format!(
                "uncertain aliases claimed complete expansion: {reply}"
            ));
        }
        let second = invoke(&env, super::action(&reply, "page")?, 0)?;
        if second.get("status") != reply.get("status") || query(&second)? != query(&reply)? {
            return Err("continuation lost incomplete or ambiguous resolution".into());
        }
        stub.finish()?;
    }
    Ok(())
}

fn controls() -> Check {
    meddra::check()?;
    let (env, stub, _) = setup(
        0,
        json!({"disable_sources":["mychem","chembl","openfda-label"]}),
    )?;
    let reply = invoke(&env, args(true), 0)?;
    if query(&reply)?.contains("Brand name") || reply.get("status") != Some(&json!("partial")) {
        return Err("disabled lookup sources silently satisfied expansion".into());
    }
    if stub
        .finish()?
        .iter()
        .any(|(_, line)| line.contains("/v1/query?"))
    {
        return Err("disabled name lookup ran".into());
    }
    let (env, stub, _) = setup(0, json!({}))?;
    for source in ["vaers", "maude"] {
        let mut words = args(true);
        words.extend(["--source".into(), source.into()]);
        invoke(&env, words, 2)?;
    }
    let first = invoke(&env, args(true), 0)?;
    snapshots(&env, &first)?;
    let mut changed = super::action(&first, "page")?;
    changed.extend(["--reaction".into(), "Changed filter".into()]);
    let refused = invoke(&env, changed, 3)?;
    if refused.to_string().contains("Brand name") {
        return Err("snapshot escaped its selectors".into());
    }
    stub.finish()?;
    bounds()?;
    Ok(())
}

fn bounds() -> Check {
    for mode in [4, 5] {
        let (env, stub, _) = setup(mode, json!({}))?;
        let reply = invoke(&env, args(true), 0)?;
        if query(&reply)?.len() > 4096
            || mode == 4 && query(&reply)?.contains("Long alias")
            || mode == 4 && !reply.to_string().contains("Left out 1 additional names")
            || mode == 5 && !query(&reply)?.contains(r#"Brand \"quoted\" \\name"#)
        {
            return Err(format!(
                "expanded names escaped the native query bound: {reply}"
            ));
        }
        stub.finish()?;
    }
    let (env, stub, _) = setup(0, json!({"drug_search_name_limit":1}))?;
    let reply = invoke(&env, args(true), 0)?;
    if query(&reply)?.contains("Brand name")
        || !reply.to_string().contains("drug_search_name_limit")
    {
        return Err("configured resolver limit was ignored or hidden".into());
    }
    stub.finish()?;
    Ok(())
}

fn snapshots(env: &Environment, first: &Value) -> Check {
    let next = super::action(first, "page")?;
    let index = next
        .iter()
        .position(|word| word == "--name-snapshot")
        .ok_or("missing name snapshot")?;
    let mut missing = next.clone();
    missing.drain(index..=index + 1);
    invoke(env, missing, 2)?;
    let mut tampered = next.clone();
    *tampered.get_mut(index + 1).ok_or("snapshot value")? = "synthetic-missing".into();
    let reply = invoke(env, tampered, 3)?;
    super::action(&reply, "restart")?;
    for source in ["mychem", "drugcentral"] {
        let mut disabled = next.clone();
        disabled.extend(["--disable-sources".into(), source.into()]);
        let reply = invoke(env, disabled, 3)?;
        super::action(&reply, "restart")?;
    }
    let mut literal = args(false);
    literal.extend([
        "--name-snapshot".into(),
        first
            .pointer("/query/name_snapshot")
            .and_then(Value::as_str)
            .ok_or("token")?
            .into(),
    ]);
    invoke(env, literal, 2)?;
    Ok(())
}

fn verify_native(requests: &[(String, String)], expression: &str) -> Check {
    let mut matching = 0;
    for (_, request) in requests {
        let target = request.split_whitespace().nth(1).ok_or("request target")?;
        if !target.starts_with("/drug/event.json?") {
            continue;
        }
        let url = url::Url::parse(&format!("https://synthetic.test{target}"))
            .map_err(|e| e.to_string())?;
        let sent = url
            .query_pairs()
            .find(|(key, _)| key == "search")
            .ok_or("search expression")?
            .1;
        if sent == expression {
            matching += 1;
        } else if sent.contains("Brand name") {
            return Err("native alias expression lost a supplied filter".into());
        }
    }
    if matching < 3 {
        return Err("reports, next page and counts did not share one native expression".into());
    }
    Ok(())
}
