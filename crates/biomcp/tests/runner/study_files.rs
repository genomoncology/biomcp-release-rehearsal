//! Feature study-files: references retain native blob facts and publisher notices.
use super::required_contract;
use super::{call, rows, seed};
use crate::runner::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    // The test reconstructs native file-reference pages from the retained revision and inspects publisher notice ownership.
    vec![transcript_trial(required_contract::STUDY_FILES.0, || {
        pages().map_err(Into::into)
    })]
}
fn entries() -> Value {
    json!(
        (0..12)
            .map(|n| {
                let name = if n == 11 { "LICENSE".into() } else { format!("data_{n}.txt") };
                json!({"name":name,
        "path":format!("public/synthetic_0/{name}"),
        "html_url":format!("https://github.com/cBioPortal/datahub/blob/master/public/synthetic_0/{name}"),
        "download_url":format!("https://raw.githubusercontent.com/cBioPortal/datahub/master/public/synthetic_0/{name}"),
        "sha":"0123456789012345678901234567890123456789","size":131,"type":"file"})
            })
            .collect::<Vec<_>>()
    )
}
pub(super) fn seed_files(replay: &Path) -> Check {
    record(replay, entries(), 200, 0)
}
fn record(replay: &Path, body: Value, status: u16, delay: u64) -> Check {
    record_raw(replay, &body.to_string(), status, delay)
}
pub(super) fn record_raw(replay: &Path, body: &str, status: u16, delay: u64) -> Check {
    let url = "https://api.github.com/repos/cBioPortal/datahub/contents/public/synthetic_0";
    let key = Sha256::digest([b"GET\0".as_slice(), url.as_bytes()].concat())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let exchange = json!(
        {"status":status,"fetched":42,"content_type":"application/json","delay_ms":delay,"retry_after_ms":null,"body":{"kind":"text",
        "value":body}}
    );
    let exchanges = if matches!(status, 429 | 500) {
        vec![exchange; 3]
    } else {
        vec![exchange]
    };
    let value = json!({"source":"cbioportal","method":"GET","url":url,"exchanges":exchanges});
    fs::write(
        replay.join("cbioportal").join(format!("{key}.json")),
        value.to_string(),
    )
    .map_err(|e| e.to_string())
}
fn pages() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env, rows(), 200)?;
    call(
        &env,
        &replay,
        &["sync", "cbioportal-studies"],
        0,
        "files/pages/1",
    )?;
    let cold = call(
        &env,
        &replay,
        &["get", "study", "synthetic_0"],
        0,
        "files/pages/2",
    )?;
    let has_files = cold
        .get("next")
        .and_then(Value::as_array)
        .is_some_and(|steps| {
            steps.iter().any(|step| {
                step.get("kind") == Some(&json!("section"))
                    && step
                        .get("args")
                        .and_then(Value::as_array)
                        .is_some_and(|args| args.iter().any(|word| word.as_str() == Some("files")))
            })
        });
    if !has_files {
        return Err("stored card lost files action".into());
    }
    record(&replay, entries(), 200, 0)?;
    let mut seen = std::collections::BTreeSet::new();
    let mut args = ["get", "study", "synthetic_0", "files"]
        .map(str::to_owned)
        .to_vec();
    for offset in [0, 5, 10] {
        let words: Vec<_> = args.iter().map(String::as_str).collect();
        let value = call(&env, &replay, &words, 0, "files/pages/3")?;
        check_page(&value, offset, &mut seen)?;
        let mcp = crate::runner::mcp::call_with_globals(
            &env,
            &args,
            false,
            &env.globals(Some(&replay), false),
        )?;
        if value != mcp {
            return Err("CLI/MCP files differ".into());
        }
        if offset < 10 {
            args = crate::runner::action(&value, "page")?;
        } else if crate::runner::action(&value, "page").is_ok() {
            return Err("last file page offered another page".into());
        }
    }
    let card = call(
        &env,
        &replay,
        &["get", "study", "synthetic_0"],
        0,
        "files/pages/4",
    )?;
    if card != cold || card.pointer("/result/license_notice").is_some() || seen.len() != 12 {
        return Err("stored card changed or walk missing".into());
    }
    Ok(())
}
fn check_page(
    value: &Value,
    offset: usize,
    seen: &mut std::collections::BTreeSet<String>,
) -> Check {
    let items = value
        .pointer("/result/items")
        .and_then(Value::as_array)
        .ok_or("file items")?;
    if value.pointer("/result/acquired") != Some(&json!(42))
        || value.pointer("/page/total") != Some(&json!(12))
        || value.pointer("/page/offset") != Some(&json!(offset))
        || items.len() != (12 - offset).min(5)
    {
        return Err(format!("native page bounds lost: {value}"));
    }
    for row in items {
        let path = row
            .get("path")
            .and_then(Value::as_str)
            .ok_or("native path")?;
        if !seen.insert(path.to_owned())
            || row.get("repositoryBlobSize") != Some(&json!(131))
            || row.get("data_type").is_some()
        {
            return Err(format!("blob identity or size changed: {row}"));
        }
    }
    if value
        .pointer("/result/license_notice")
        .and_then(Value::as_str)
        != Some("https://github.com/cBioPortal/datahub/blob/master/public/synthetic_0/LICENSE")
    {
        return Err(format!("publisher notice missing: {value}"));
    }
    Ok(())
}
