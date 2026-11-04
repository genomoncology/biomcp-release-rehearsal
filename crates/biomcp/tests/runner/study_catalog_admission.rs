//! Feature study-catalog: native admission determines replacement without losing the last good catalog.
use super::{rows, seed as seed_catalog};
use crate::runner::{configuration::invoke, launch::Environment, transcript::trial};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial("study-catalog/native-empty-publishes-empty", || {
            empty().map_err(Into::into)
        }),
        trial(
            "study-catalog/refused-refresh-retains-active-catalog",
            || refused().map_err(Into::into),
        ),
        trial(
            "study-catalog/mixed-refresh-publishes-accepted-identities",
            || mixed().map_err(Into::into),
        ),
    ]
}
fn seed(env: &Environment, body: Value, status: u16) -> Result<PathBuf, String> {
    let replay = seed_catalog(env, body, status)?;
    if crate::runner::transcript::current().is_some() {
        let owner = tempfile::tempdir().map_err(|e| e.to_string())?;
        let recorded = owner.path().join("recordings/cbioportal");
        fs::create_dir_all(&recorded).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(replay.join("cbioportal")).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            fs::copy(entry.path(), recorded.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
        crate::runner::network_stub::retain_replay_owner(&owner).map_err(|e| e.to_string())?;
    }
    Ok(replay)
}
fn call(env: &Environment, replay: &Path, args: &[&str]) -> Result<Value, String> {
    call_exit(env, replay, args, 0)
}
fn call_exit(env: &Environment, replay: &Path, args: &[&str], exit: i32) -> Result<Value, String> {
    let mut command = env.globals(Some(replay), false);
    command.extend(args.iter().map(|word| (*word).into()));
    let cli = invoke(env, command, exit)?;
    let words = args
        .iter()
        .map(|word| (*word).to_owned())
        .collect::<Vec<_>>();
    let mcp = crate::runner::mcp::call_with_globals(
        env,
        &words,
        exit > 1,
        &env.globals(Some(replay), false),
    )?;
    for pointer in [
        "/status",
        "/result/rows",
        "/result/native_rows",
        "/sources/0/state",
        "/problems",
    ] {
        if cli.pointer(pointer) != mcp.pointer(pointer) {
            return Err(format!("CLI/MCP differ at {pointer}: {cli} / {mcp}"));
        }
    }
    // MCP ran last and owns the active snapshot when a sync crosses a clock second.
    Ok(mcp)
}
fn check(value: &Value, status: &str, rows: u64, native: u64, state: &str) -> Check {
    for (pointer, expected) in [
        ("/status", json!(status)),
        ("/result/rows", json!(rows)),
        ("/result/native_rows", json!(native)),
        ("/sources/0/state", json!(state)),
    ] {
        if value.pointer(pointer) != Some(&expected) {
            return Err(format!("unexpected {pointer}: {value}"));
        }
    }
    Ok(())
}
fn rejected() -> Result<Value, String> {
    let mut body = rows();
    for row in body.as_array_mut().ok_or("synthetic rows")? {
        *row.get_mut("studyId").ok_or("study identity")? = json!("invalid identity with spaces");
    }
    Ok(body)
}
fn empty() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env, rows(), 200)?;
    call(&env, &replay, &["sync", "cbioportal-studies"])?;
    seed(&env, json!([]), 200)?;
    let empty = call(&env, &replay, &["sync", "cbioportal-studies"])?;
    check(&empty, "complete", 0, 0, "ok")?;
    let search = call(&env, &replay, &["search", "study", "atlas"])?;
    if search.pointer("/page/total") != Some(&json!(0))
        || search.get("status") != Some(&json!("complete"))
    {
        return Err(format!("empty native catalog did not activate: {search}"));
    }
    Ok(())
}
fn refused() -> Check {
    for prior in [true, false] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let replay = seed(&env, rows(), 200)?;
        let old = prior
            .then(|| call(&env, &replay, &["sync", "cbioportal-studies"]))
            .transpose()?;
        let native_rows = if prior { 3 } else { 12 };
        let mut body = rejected()?;
        body.as_array_mut()
            .ok_or("synthetic rows")?
            .truncate(native_rows as usize);
        seed(&env, body, 200)?;
        let refusal = call(&env, &replay, &["sync", "cbioportal-studies"])?;
        if let Some(old) = &old
            && (refusal.pointer("/result/rows") != Some(&json!(12))
                || refusal.pointer("/result/snapshot") != old.pointer("/result/snapshot"))
        {
            return Err(format!(
                "refused refresh replaced last good rows or snapshot: {refusal}"
            ));
        }
        check(
            &refusal,
            "partial",
            if prior { 12 } else { 0 },
            native_rows,
            if prior { "stale" } else { "ok" },
        )?;
        if refusal
            .get("problems")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(native_rows as usize)
        {
            return Err(format!("native refusals were lost: {refusal}"));
        }
        if old.is_some() {
            let found = call(&env, &replay, &["search", "study", "atlas"])?;
            if found.pointer("/page/total") != Some(&json!(12)) {
                return Err(format!("refused refresh lost active rows: {found}"));
            }
        } else {
            if refusal.pointer("/result/snapshot").is_some() {
                return Err(format!("refused refresh invented a snapshot: {refusal}"));
            }
            let missing = call_exit(&env, &replay, &["search", "study", "atlas"], 4)?;
            if missing.get("result").is_some()
                || missing.pointer("/sources/0/state") != Some(&json!("syncing"))
            {
                return Err(format!("refused refresh activated a catalog: {missing}"));
            }
        }
    }
    Ok(())
}
fn mixed() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let mut body = rejected()?;
    *body.get_mut(0).ok_or("refused row")? = rows().get(0).ok_or("accepted row")?.clone();
    let replay = seed(&env, body, 200)?;
    let receipt = call(&env, &replay, &["sync", "cbioportal-studies"])?;
    check(&receipt, "partial", 1, 12, "ok")?;
    let found = call(&env, &replay, &["search", "study", "atlas"])?;
    if found.pointer("/page/total") != Some(&json!(1))
        || receipt
            .get("problems")
            .and_then(Value::as_array)
            .map(Vec::len)
            != Some(11)
    {
        return Err(format!(
            "mixed native rows lost their accepted identity: {receipt} / {found}"
        ));
    }
    Ok(())
}
