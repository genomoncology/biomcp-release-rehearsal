//! Features pathway-wikipathways and diagnostic-cdx: external locks retain typed failures.
use super::super::{
    diagnostic_cdx, launch::Environment, mcp::request, publication::Peer, schema::Registry,
    wikipathways,
};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial(
            "pathway-wikipathways/held-store-lock-names-local-failure",
            || locked(false).map_err(Into::into),
        ),
        transcript_trial("diagnostic-cdx/held-store-lock-names-local-failure", || {
            locked(true).map_err(Into::into)
        }),
    ]
}

fn locked(cdx: bool) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let result = locked_with_environment(cdx, &env);
    result.map_err(
        |error| match biomcp_test_support::evidence::retain(env.root()) {
            Ok(root) => format!("{error}; retained store-lock fixture={}", root.display()),
            Err(retention) => format!("{error}; fixture retention failed: {retention}"),
        },
    )
}

fn locked_with_environment(cdx: bool, env: &Environment) -> Result<(), String> {
    let schemas = Registry::embedded()?;
    let replay = if cdx {
        diagnostic_cdx::seed(env)?
    } else {
        wikipathways::seed(
            env,
            wikipathways::catalog(true, false),
            wikipathways::catalog(false, false),
            200,
        )?
    };
    let repeated = env.root().join("repeated-replies");
    super::recordings(&replay, &repeated)?;
    let mut peer = Peer::with_globals(env, &env.globals(Some(&repeated), false))?;
    let source = if cdx { "fda-cdx" } else { "wikipathways" };
    let sync = vec!["sync", source];
    let initial = call("initial-sync", &mut peer, &schemas, &sync)?;
    if initial.pointer("/result/isError") != Some(&json!(false))
        || initial
            .pointer("/result/structuredContent/local_error")
            .is_some()
    {
        return Err(format!("initial sync failed: {initial}"));
    }
    let db = rusqlite::Connection::open(env.root().join("case-cache/replay.sqlite"))
        .map_err(|e| e.to_string())?;
    db.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| e.to_string())?;
    if !cdx {
        let reader = call(
            "held-lock-reader",
            &mut peer,
            &schemas,
            &["search", "pathway", "Atlas", "--source", source],
        )?;
        if reader.pointer("/result/isError") != Some(&json!(false))
            || reader
                .pointer("/result/structuredContent/result/items/0")
                .is_none()
            || reader
                .pointer("/result/structuredContent/local_error")
                .is_some()
        {
            return Err(format!(
                "reader discarded fetched catalog under publication contention: {reader}"
            ));
        }
        if reader.pointer("/result/structuredContent/sources/0/state") != Some(&json!("ok")) {
            return Err(format!("reader lost its checked provider state: {reader}"));
        }
    }
    failure(&call("held-lock-sync", &mut peer, &schemas, &sync)?)?;
    let mut args = env.globals(Some(&repeated), false);
    args.extend(sync.iter().map(|word| (*word).to_owned()));
    let cli = super::super::configuration::invoke(env, args.clone(), 5)?;
    schemas.validate(&cli, &args, false)?;
    if cli.get("local_error") != Some(&json!("busy")) || cli.get("status") != Some(&json!("failed"))
    {
        return Err(format!("fresh CLI erased cache failure: {cli}"));
    }
    db.execute_batch("ROLLBACK").map_err(|e| e.to_string())?;
    peer.finish()
}

fn call(phase: &str, peer: &mut Peer, schemas: &Registry, args: &[&str]) -> Result<Value, String> {
    let started = std::time::Instant::now();
    peer.send(&request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp", "arguments":{"args":args}}),
    ))?;
    let reply = peer
        .next()
        .map_err(|error| format!("{phase} {args:?}: {error}"))?;
    use std::io::Write;
    writeln!(
        std::io::stderr(),
        "STORE_LOCK_MCP phase={phase} elapsed={:?}",
        started.elapsed()
    )
    .map_err(|e| e.to_string())?;
    let content = reply
        .pointer("/result/structuredContent")
        .ok_or("content missing")?;
    let words = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    schemas.validate(content, &words, false)?;
    if let Some(kind) = content.get("local_error") {
        let text = reply
            .pointer("/result/content/0/text")
            .and_then(Value::as_str)
            .ok_or("MCP content text missing")?;
        use std::io::Write;
        writeln!(
            std::io::stdout(),
            "LOCAL_ERROR_MCP_BYTES {} {kind} {}",
            args.join(" "),
            text.len()
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(reply)
}

fn failure(reply: &Value) -> Result<(), String> {
    if reply.pointer("/result/isError") != Some(&json!(true))
        || reply.pointer("/result/structuredContent/local_error") != Some(&json!("busy"))
        || reply.pointer("/result/structuredContent/status") != Some(&json!("failed"))
    {
        return Err(format!(
            "held external writer lacks typed local failure: {reply}"
        ));
    }
    Ok(())
}
