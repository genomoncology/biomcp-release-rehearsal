//! Feature study-catalog: stored literal search never synchronizes implicitly.
use super::required_contract;
use super::{configuration::invoke, launch::Environment};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
type Check = Result<(), String>;
const URL: &str = "https://www.cbioportal.org/api/studies?projection=DETAILED";
pub(super) fn trials() -> Vec<Trial> {
    // The test removes the replay source before traversing the runtime catalog revision and validates immutable store ownership.
    // Replacing the recording after the accepted sync is required: CLI case steps each restart replay exchanges.
    let mut trials = vec![
        transcript_trial(
            "study-failures/failed-refresh-preserves-published-catalog",
            || replacement().map_err(Into::into),
        ),
        transcript_trial(required_contract::STUDY_SEARCH.0, || {
            search().map_err(Into::into)
        }),
    ];
    trials.extend(admission::trials());
    trials.extend(study_files::trials());
    trials.extend(study_get::trials());
    trials
}

#[path = "study_catalog_admission.rs"]
mod admission;

#[path = "study_files.rs"]
mod study_files;
#[path = "study_get.rs"]
mod study_get;

pub(super) fn rows() -> Value {
    json!(
        (0..12)
            .map(|n| json!({"studyId":format!("synthetic_{n}"),"name":format!("Synthetic Atlas {n}"),
        "description":"Literal mixed metadata","cancerTypeId":if n < 8 {"brca"} else {"mixed"},"allSampleCount":n,"referenceGenome":"hg19",
        "sequencedSampleCount":n,"cnaSampleCount":0,"structuralVariantCount":0,"treatmentCount":0,"resourceCounts":[],
        "importDate":"2026-08-20 21:46:46","groups":"","readPermission":true,"publicStudy":true}))
            .collect::<Vec<_>>()
    )
}
pub(super) fn seed(env: &Environment, body: Value, status: u16) -> Result<PathBuf, String> {
    let dir = env.root().join("replay/cbioportal");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // The recording filename belongs to the same canonical GET URL as its body.
    let key = Sha256::digest(format!("GET\0{URL}").as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let recording = json!(
        {"source":"cbioportal","method":"GET","url":URL,"exchanges":[{"status":status,"content_type":"application/json","delay_ms":0,
        "retry_after_ms":null,"body":{"kind":"text","value":body.to_string()}}]}
    );
    fs::write(dir.join(format!("{key}.json")), recording.to_string()).map_err(|e| e.to_string())?;
    if crate::runner::transcript::current().is_some() {
        crate::runner::network_stub::retain_replay_owner(env.shared_root().as_ref())
            .map_err(|e| e.to_string())?;
    }
    Ok(env.root().join("replay"))
}
pub(super) fn call(
    env: &Environment,
    replay: &Path,
    args: &[&str],
    exit: i32,
    phase: &str,
) -> Result<Value, String> {
    let mut command = env.globals(Some(replay), false);
    command.extend(args.iter().map(|s| (*s).into()));
    let value = invoke(env, command, exit)?;
    if value.get("schema") == Some(&json!("biomcp.study-page/5"))
        && value
            .pointer("/query/limit")
            .and_then(Value::as_u64)
            .is_some_and(|n| n <= 5)
        && serde_json::to_vec(&value).map_err(|e| e.to_string())?.len() + 1
            > crate::runner::output_limit("study-sync-cli")?
    {
        return Err(
            "study header page exceeds retained-catalog measurement plus fifteen percent".into(),
        );
    }
    budget(&value, phase, "cli")?;
    let words = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    let mcp =
        super::mcp::call_with_globals(env, &words, exit > 1, &env.globals(Some(replay), false))?;
    budget(&mcp, phase, "mcp")?;
    Ok(value)
}
#[path = "study_budgets.rs"]
mod budgets;
pub(super) fn budget(value: &Value, phase: &str, surface: &str) -> Check {
    budgets::check(value, phase, surface)
}

fn search() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env, rows(), 200)?;
    let receipt = call(
        &env,
        &replay,
        &["sync", "cbioportal-studies"],
        0,
        "catalog/search/1",
    )?;
    if receipt.pointer("/result/rows") != Some(&json!(12))
        || receipt
            .pointer("/result/snapshot/acquired")
            .and_then(Value::as_u64)
            .is_none()
    {
        return Err(format!("receipt: {receipt}"));
    }
    fs::remove_dir_all(replay.join("cbioportal")).map_err(|e| e.to_string())?;
    let first = call(
        &env,
        &replay,
        &["search", "study", "ATLAS"],
        0,
        "catalog/search/2",
    )?;
    if first.pointer("/result/items/0/studyId") != Some(&json!("synthetic_0"))
        || first.pointer("/page/returned") != Some(&json!(5))
        || first.pointer("/page/total") != Some(&json!(12))
    {
        return Err(format!("literal search: {first}"));
    }
    page_walk(&env, &replay, &first)?;
    for (args, total) in [
        (vec!["search", "study", "--cancer-type", "brca"], 8),
        (vec!["search", "study", "--cancer-type", "BRCA"], 0),
        (
            vec!["search", "study", "synthetic_11", "--cancer-type", "brca"],
            0,
        ),
        (
            vec![
                "search",
                "study",
                "atlas",
                "--cancer-type",
                "mixed",
                "--limit",
                "50",
            ],
            4,
        ),
        (vec!["search", "study", "fabricated"], 0),
        (vec!["search", "study", "synthetic_11", "--limit", "1"], 1),
    ] {
        let value = call(&env, &replay, &args, 0, "catalog/search/3")?;
        if value.pointer("/page/total") != Some(&json!(total)) {
            return Err(format!("selectors: {value}"));
        }
    }
    let pool = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("pool")?;
    let changed = call(
        &env,
        &replay,
        &["search", "study", "other", "--pool", pool],
        2,
        "catalog/search/4",
    )?;
    if changed.get("result").is_some() {
        return Err("changed selectors reused a pool".into());
    }
    Ok(())
}

fn page_walk(env: &Environment, replay: &Path, first: &Value) -> Check {
    let mut current = first.clone();
    let mut ids = Vec::new();
    loop {
        ids.extend(
            current
                .pointer("/result/items")
                .and_then(Value::as_array)
                .ok_or("items")?
                .iter()
                .filter_map(|v| v.get("studyId").and_then(Value::as_str))
                .map(str::to_owned),
        );
        let next = current
            .get("next")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().find(|v| v.get("kind") == Some(&json!("page"))));
        let Some(next) = next else { break };
        let args = next
            .get("args")
            .and_then(Value::as_array)
            .ok_or("args")?
            .iter()
            .map(|v| v.as_str().ok_or("arg"))
            .collect::<Result<Vec<_>, _>>()?;
        current = call(env, replay, &args, 0, "catalog/page_walk/1")?;
    }
    if ids
        != (0..12)
            .map(|n| format!("synthetic_{n}"))
            .collect::<Vec<_>>()
    {
        return Err("page walk changed order or lost rows".into());
    }
    Ok(())
}

fn replacement() -> Check {
    for fault in ["conflict", "oversized"] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let replay = seed(&env, rows(), 200)?;
        call(
            &env,
            &replay,
            &["sync", "cbioportal-studies"],
            0,
            "catalog/replacement/1",
        )?;
        let mut body = rows();
        if fault == "conflict" {
            let row = body.get(0).ok_or("row")?.clone();
            body.as_array_mut().ok_or("rows")?.push(row);
        }
        if fault == "oversized" {
            *body.pointer_mut("/0/description").ok_or("description")? =
                json!("x".repeat(1024 * 1024));
        }
        seed(&env, body, 200)?;
        let failure = call(
            &env,
            &replay,
            &["sync", "cbioportal-studies"],
            4,
            "catalog/replacement/2",
        )?;
        if failure.pointer("/sources/0/state") != Some(&json!("failed"))
            || failure.pointer("/sources/0/use") != Some(&json!("attribution"))
            || failure.pointer("/sources/0/terms").is_none()
        {
            return Err(format!(
                "sync fault lost terms or source outcome: {failure}"
            ));
        }
        let found = call(
            &env,
            &replay,
            &["search", "study", "atlas"],
            0,
            "catalog/replacement/3",
        )?;
        if found.pointer("/page/total") != Some(&json!(12)) {
            return Err(format!("failed replacement lost old rows: {found}"));
        }
        for group in ["cbioportal", "restricted"] {
            let skipped = call(
                &env,
                &replay,
                &["search", "study", "atlas", "--disable-sources", group],
                4,
                "catalog/replacement/4",
            )?;
            if skipped.get("result").is_some()
                || skipped.pointer("/sources/0/reason") != Some(&json!("disabled"))
            {
                return Err(format!("disabled read: {skipped}"));
            }
            call(
                &env,
                &replay,
                &["sync", "cbioportal-studies", "--disable-sources", group],
                4,
                "catalog/replacement/5",
            )?;
        }
    }
    Ok(())
}
