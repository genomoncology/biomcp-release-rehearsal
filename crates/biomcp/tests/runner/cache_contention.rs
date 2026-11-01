//! A checked provider reply survives a competing cache writer.
use super::{
    configuration::{cached, invoke, root},
    launch::Environment,
    mcp::request,
    publication::Peer,
};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};

mod article_publish;
mod record_publish;
mod retention;
mod store_locks;

pub(super) fn trials() -> Vec<Trial> {
    let mut trials = vec![
        transcript_trial(
            "cache-resilience/checked-gene-survives-write-contention",
            || checked_gene().map_err(Into::into),
        ),
        transcript_trial(
            "cache-resilience/checked-protein-command-survives-write-contention",
            || checked_protein().map_err(Into::into),
        ),
    ];
    trials.push(transcript_trial(
        "cache-resilience/stale-replies-survive-unrelated-write-until-grace",
        || retention::stale().map_err(Into::into),
    ));
    trials.push(transcript_trial(
        "article-search/current-store-opens-without-write-lock",
        || retention::open_reader().map_err(Into::into),
    ));
    trials.push(transcript_trial(
        "adverse-event-get/late-store-lock-keeps-fetched-faers-record",
        || record_publish::check(false, None).map_err(Into::into),
    ));
    trials.push(transcript_trial(
        "maude-reports/late-store-lock-keeps-fetched-maude-record",
        || record_publish::check(true, None).map_err(Into::into),
    ));
    for (device, section) in [
        (false, "drugs"),
        (false, "reactions"),
        (false, "unii"),
        (false, "rxcui"),
        (false, "product-ndc"),
        (false, "package-ndc"),
        (false, "spl-id"),
        (false, "spl-set-id"),
        (false, "application-number"),
        (true, "devices"),
        (true, "narrative"),
    ] {
        trials.push(transcript_trial(
            format!("cache-resilience/late-store-lock-keeps-fetched-{section}"),
            move || record_publish::check(device, Some(section)).map_err(Into::into),
        ));
    }
    trials.extend(store_locks::trials());
    trials.push(transcript_trial(
        "article-search/late-store-lock-keeps-fetched-items",
        || article_publish::check().map_err(Into::into),
    ));
    trials
}

fn checked_gene() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let records = env.root().join("records");
    recordings(&root(), &records)?;
    let mut peer = Peer::with_globals(&env, &env.globals(Some(&records), false))?;
    let frame = request(
        json!(1),
        "tools/call",
        json!({
            "name":"biomcp", "arguments":{"args":["get","gene","BRAF"]}
        }),
    );
    peer.send(&frame)?;
    let initial = content(peer.next()?)?;
    let db = rusqlite::Connection::open(env.root().join("case-cache/replay.sqlite"))
        .map_err(|e| e.to_string())?;
    db.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| e.to_string())?;
    let mut fresh = frame;
    fresh
        .pointer_mut("/params/arguments/args")
        .and_then(Value::as_array_mut)
        .ok_or("command args missing")?
        .push(json!("--fresh"));
    peer.send(&fresh)?;
    let answer = content(peer.next()?)?;
    if answer.pointer("/result/items") != initial.pointer("/result/items") {
        return Err(format!(
            "cache contention changed the checked gene: {answer}"
        ));
    }
    let sources = answer
        .get("sources")
        .and_then(Value::as_array)
        .ok_or("sources missing")?;
    let mygene = sources
        .iter()
        .find(|s| s.get("source") == Some(&json!("mygene")))
        .ok_or("mygene outcome missing")?;
    if mygene.get("state") != Some(&json!("ok"))
        || mygene.pointer("/notices/0/kind") != Some(&json!("cache-problem"))
    {
        return Err(format!("checked source lost its cache problem: {answer}"));
    }
    db.execute_batch("ROLLBACK").map_err(|e| e.to_string())?;
    peer.finish()
}

fn content(reply: Value) -> Result<Value, String> {
    if reply.pointer("/result/isError") != Some(&json!(false)) {
        return Err(format!("checked gene command failed: {reply}"));
    }
    reply
        .pointer("/result/structuredContent")
        .cloned()
        .ok_or("content missing".into())
}

fn recordings(from: &std::path::Path, to: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            recordings(&path, &target)?;
        } else if path.extension().is_some_and(|v| v == "json") {
            let mut value: Value =
                serde_json::from_slice(&std::fs::read(&path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            super::response_cases::cache_acquisition(
                &mut value,
                super::response_cases::CACHE_ACQUIRED,
            );
            if let Some(exchanges) = value.get_mut("exchanges").and_then(Value::as_array_mut) {
                let copy = exchanges.clone();
                exchanges.extend(copy);
            }
            std::fs::write(
                target,
                serde_json::to_vec(&value).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn checked_protein() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let case = env.root().join("case");
    let records = env.root().join("known-recordings");
    super::response_cases::cache_capture(
        "protein/synthetic",
        &records,
        super::response_cases::CACHE_ACQUIRED,
    )?;
    let args = cached(&["get", "protein", "P00533"], &case, Some(&records));
    let initial = invoke(&env, args.clone(), 0)?;
    let db = rusqlite::Connection::open(case.join("replay.sqlite")).map_err(|e| e.to_string())?;
    db.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| e.to_string())?;
    let mut fresh = args;
    fresh.push("--fresh".into());
    let answer = invoke(&env, fresh, 0)?;
    if answer.get("result") != initial.get("result")
        || answer.get("status") != Some(&json!("complete"))
        || answer.pointer("/sources/0/state") != Some(&json!("ok"))
        || answer.pointer("/sources/0/notices/0/kind") != Some(&json!("cache-problem"))
        || answer.pointer("/sources/0/notices/0/reason") != Some(&json!("busy"))
    {
        return Err(format!(
            "CLI discarded a checked reply under cache contention: {answer}"
        ));
    }
    Ok(())
}
