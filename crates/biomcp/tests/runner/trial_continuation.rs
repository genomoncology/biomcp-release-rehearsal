//! A rejected registry page has explicit fresh recovery through CLI and stdio MCP.
use super::{configuration::invoke, launch::Environment, mcp, network_stub::Stub};
use serde_json::{Value, json};
use std::fs;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    [false, true]
        .into_iter()
        .flat_map(|wrapped| [false, true].map(move |mcp| (wrapped, mcp)))
        .map(|(wrapped, mcp)| {
            super::transcript::trial(
                format!(
                    "trial-search/rejected-continuation-{}-{}",
                    if wrapped { "broader" } else { "native" },
                    if mcp { "mcp" } else { "cli" }
                ),
                move || check(wrapped, mcp).map_err(Into::into),
            )
        })
        .collect()
}

fn setup(wrapped: bool) -> Result<(Environment, Stub), String> {
    let mut first_pages = 0;
    let stub = Stub::for_source_status(
        "clinicaltrials.gov",
        "application/json",
        move |request| {
            if request.contains("/version") {
                return Ok(json!({"dataTimestamp":"2026-11-10T00:00:00Z"}).to_string());
            }
            if !request.contains("/studies?") {
                return Err(format!("unexpected registry request: {request}"));
            }
            if request.contains("pageToken=old-cursor") {
                return Ok("private registry refusal must not be echoed".into());
            }
            if wrapped && !request.contains("query.spons=") {
                return Ok(json!({"totalCount":0,"studies":[]}).to_string());
            }
            let later = request.contains("pageToken=fresh-cursor");
            let mut page = json!({"totalCount":2,"nextPageToken":null,"studies":[{"protocolSection":{
                "identificationModule":{"nctId":if later {"NCT00000002"} else {"NCT00000001"},
                    "briefTitle":"Synthetic registry study"},
                "statusModule":{"overallStatus":"RECRUITING"}
            }}]});
            if !later {
                first_pages += 1;
                *page
                    .get_mut("nextPageToken")
                    .ok_or("synthetic page token field")? = json!(if first_pages == 1 {
                    "old-cursor"
                } else {
                    "fresh-cursor"
                });
            }
            Ok(page.to_string())
        },
        |request| {
            if request.contains("pageToken=old-cursor") {
                400
            } else {
                200
            }
        },
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("registry-ca.pem");
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    stub.retain(env.shared_root());
    env.config(
        &json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,
        "disable_sources":["chembl","openfda-label"]}),
    )
    .map_err(|e| e.to_string())?;
    Ok((env, stub))
}

fn call(env: &Environment, args: &[String], meta: bool, error: bool) -> Result<Value, String> {
    if meta {
        mcp::call_with_globals(env, args, error, &[])
    } else {
        invoke(env, args.to_vec(), if error { 2 } else { 0 })
    }
}

fn check(wrapped: bool, meta: bool) -> Result<(), String> {
    let (env, stub) = setup(wrapped)?;
    let config = env
        .root()
        .join("config/biomcp/config.json")
        .to_string_lossy()
        .into_owned();
    let mut args = [
        "--config",
        &config,
        "--disable-sources",
        "mychem",
        "search",
        "trial",
        "--source",
        "ctgov",
        "--sponsor",
        "Synthetic Registry",
        "--status",
        "RECRUITING",
        "--phase",
        "1",
        "--has-results",
        "--limit",
        "1",
        "--exact-term",
    ]
    .map(str::to_owned)
    .to_vec();
    if meta {
        // Stdio owns its default configuration; tool calls cannot select another config file.
        args.drain(..2);
    }
    let first = call(&env, &args, meta, false)?;
    let old = super::action(&first, "page")?;
    let cached = call(&env, &args, meta, false)?;
    if super::action(&cached, "page")? != old {
        return Err(format!(
            "first-page cache was bypassed before explicit recovery: {cached}"
        ));
    }
    let rejected = call(&env, &old, meta, true)?;
    if rejected.pointer("/error/option") != Some(&json!("--next-page"))
        || rejected.pointer("/error/source") != Some(&json!("ctgov"))
        || rejected.pointer("/error/code") != Some(&json!("usage.arguments"))
        || rejected
            .pointer("/error/message")
            .and_then(Value::as_str)
            .is_none_or(|message| {
                !message.starts_with("ClinicalTrials.gov rejected this continuation request")
            })
        || rejected.to_string().contains("old-cursor")
        || rejected.to_string().contains("private registry refusal")
    {
        return Err(format!(
            "rejected continuation lost specific safe guidance: {rejected}"
        ));
    }
    recover(&env, &first, &rejected, &args, meta)?;
    let requests = stub.finish()?;
    let rejected_count = requests
        .iter()
        .filter(|(_, r)| r.contains("pageToken=old-cursor"))
        .count();
    if rejected_count != 1 {
        return Err(format!(
            "continuation was retried automatically: {requests:?}"
        ));
    }
    Ok(())
}

fn recover(
    env: &Environment,
    first: &Value,
    error: &Value,
    original: &[String],
    meta: bool,
) -> Result<(), String> {
    let restart = super::action(error, "configure")?;
    if restart
        .iter()
        .any(|word| matches!(word.as_str(), "--next-page" | "--offset"))
        || !restart.iter().any(|word| word == "--fresh")
        || !original
            .windows(2)
            .filter(|words| {
                words
                    .first()
                    .is_some_and(|word| matches!(word.as_str(), "--config" | "--disable-sources"))
            })
            .all(|words| restart.windows(2).any(|pair| pair == words))
    {
        return Err(format!(
            "restart changed globals or retained a cursor: {restart:?}"
        ));
    }
    let restarted = call(env, &restart, meta, false)?;
    if restarted.get("query") != first.get("query")
        || restarted.pointer("/result/items/0/nct_id") != Some(&json!("NCT00000001"))
    {
        return Err(format!(
            "fresh restart changed filters or records: {restarted}"
        ));
    }
    let page = call(env, &super::action(&restarted, "page")?, meta, false)?;
    if page.pointer("/status") != Some(&json!("complete"))
        || page.pointer("/page/offset") != Some(&json!(1))
        || page.pointer("/result/items/0/nct_id") != Some(&json!("NCT00000002"))
    {
        return Err(format!(
            "new continuation did not follow after fresh recovery: {page}"
        ));
    }
    Ok(())
}
