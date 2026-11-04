//! NCI security boundary through real isolated CLI commands and a synthetic TLS peer.
#![expect(
    clippy::indexing_slicing,
    reason = "Synthetic recording mutations fail directly on fixture shape drift"
)]
use super::{launch::Environment, network_stub::Stub};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, path::Path};
mod biomarker;

mod organization;

const KEY: &str = "synthetic-nci-stored-key";
pub(super) fn trials() -> Vec<Trial> {
    std::iter::once(super::transcript::trial(
        "trial-nci/nci/wire-and-failures",
        || wire().map_err(Into::into),
    ))
    .chain(organization::trials())
    .chain(biomarker::trials())
    .collect()
}

pub(super) fn invoke(env: &Environment, args: &[String], exit: i32) -> Result<Value, String> {
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), args)
        .env("NCI_API_KEY", "synthetic-nci-env-key")
        .output()
        .map_err(|e| e.to_string())?;
    let all = [output.stdout.as_slice(), output.stderr.as_slice()].concat();
    if all.windows(KEY.len()).any(|s| s == KEY.as_bytes())
        || all.windows(21).any(|s| s == b"synthetic-nci-env-key")
    {
        return Err("credential leaked into command output".into());
    }
    if output.status.code() != Some(exit) || !output.stderr.is_empty() {
        return Err(format!(
            "wrong NCI command exit {:?}, expected {exit}",
            output.status.code()
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    super::schema::Registry::embedded()?.validate(&value, args, exit == 0)?;
    Ok(value)
}

fn reply(request: &str) -> Result<String, String> {
    let card = json!(
        {"nci_id":"NCI-2026-00001","nct_id":"NCT02296125","brief_title":"Synthetic trial",
        "current_trial_status":"Temporarily Closed to Accrual","phase":"II/III","brief_summary":"Original source summary Ω",
        "diseases":[{"name":"Synthetic disease","nci_thesaurus_concept_id":"C3224"}]}
    );
    Ok(if request.starts_with("GET /api/v2/trials/NCT02296125 ") {
        card
    } else {
        json!({"total":7,"data":[card]})
    }
    .to_string())
}

fn globals(args: &[&str], flag: &str, path: &Path) -> Vec<String> {
    args.iter()
        .map(|s| (*s).to_owned())
        .chain([flag.to_owned(), path.to_string_lossy().into_owned()])
        .collect()
}

fn wire() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let stub = Stub::for_source("clinicaltrialsapi.cancer.gov", "application/json", reply)?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,"keys":{"nci_api_key":KEY}}))
        .map_err(|e| e.to_string())?;
    let search = [
        "search",
        "trial",
        "--source",
        "nci",
        "--condition",
        "C3224",
        "--limit",
        "1",
    ];
    let get = ["get", "trial", "NCT02296125", "all", "--source", "nci"];
    let search_dir = env.root().join("search");
    let get_dir = env.root().join("get");
    let page = invoke(&env, &globals(&search, "--record", &search_dir), 0)?;
    if page.pointer("/sources/0/count") != Some(&json!(7))
        || page.pointer("/result/items/0/current_trial_status")
            != Some(&json!("Temporarily Closed to Accrual"))
    {
        return Err("native NCI status or count changed".into());
    }
    let card = invoke(&env, &globals(&get, "--record", &get_dir), 0)?;
    if card.pointer("/result/items/0/card/brief_summary/text")
        != Some(&json!("Original source summary Ω"))
    {
        return Err("native exact card missing".into());
    }
    let requests = stub.finish()?;
    if requests.len() != 2 {
        return Err("NCI made extra requests or a registry fallback".into());
    }
    for (_, request) in &requests {
        let header = request.lines().find_map(|line| {
            line.split_once(':')
                .filter(|(name, _)| name.eq_ignore_ascii_case("x-api-key"))
                .map(|(_, v)| v.trim())
        });
        if header != Some(KEY) {
            return Err("stored NCI key did not override environment at wire".into());
        }
    }
    if !requests[0]
        .1
        .contains("diseases.nci_thesaurus_concept_id=C3224")
        || requests[0].1.contains("keyword=")
    {
        return Err("disease filter degraded to keywords".into());
    }
    for directory in [&search_dir, &get_dir] {
        for file in fs::read_dir(directory.join("nci")).map_err(|e| e.to_string())? {
            let path = file.map_err(|e| e.to_string())?.path();
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if bytes.windows(KEY.len()).any(|s| s == KEY.as_bytes()) {
                return Err("credential entered recording".into());
            }
        }
    }
    failures(&env, &search, &search_dir)?;
    policies(&env, &get, &get_dir)
}

fn policies(env: &Environment, get: &[&str], dir: &Path) -> Result<(), String> {
    let mut args = globals(get, "--replay", dir);
    args.extend(["--disable-sources".into(), "nci".into()]);
    let out = invoke(env, &args, 4)?;
    if out.pointer("/sources/0/reason") != Some(&json!("disabled"))
        || out.pointer("/result/items/0/card").is_some()
    {
        return Err("disabled NCI source restored a recording".into());
    }
    env.config(&json!({})).map_err(|e| e.to_string())?;
    // This launch omits the synthetic environment key, proving the required-key guard precedes replay.
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &globals(get, "--replay", dir))
        .output()
        .map_err(|e| e.to_string())?;
    let out: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if output.status.code() != Some(4)
        || out.pointer("/sources/0/reason") != Some(&json!("no-key"))
        || out.pointer("/result/items/0/card").is_some()
    {
        return Err("missing NCI key restored a recording".into());
    }
    Ok(())
}

fn failures(env: &Environment, search: &[&str], dir: &Path) -> Result<(), String> {
    let entry = fs::read_dir(dir.join("nci"))
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("missing capture")?
        .map_err(|e| e.to_string())?
        .path();
    let original: Value = serde_json::from_slice(&fs::read(&entry).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    for (kind, reason) in [
        (401, "rejected"),
        (429, "rate-limited"),
        (500, "unavailable"),
        (0, "bad-reply"),
        (1, "deadline-exceeded"),
        (2, "bad-reply"),
        (3, "bad-reply"),
    ] {
        let target = env.root().join(format!("failure-{kind}"));
        fs::create_dir_all(target.join("nci")).map_err(|e| e.to_string())?;
        let mut record = original.clone();
        let mut exchange = record["exchanges"][0].clone();
        exchange["delay_ms"] = json!(0);
        if kind >= 400 {
            exchange["status"] = json!(kind);
            exchange["retry_after_ms"] = json!(1);
        } else if kind == 0 {
            exchange["body"] = json!({"kind":"text","value":"{\"data\":"});
        } else if kind == 2 {
            let row = exchange["body"]["value"]["data"][0].clone();
            exchange["body"]["value"]["data"] = json!([row.clone(), row]);
        } else if kind == 3 {
            exchange["body"] = json!({"kind":"json","value":{"error":"synthetic source failure"}});
        } else {
            exchange["delay_ms"] = json!(30001);
        }
        record["exchanges"] = json!(vec![
            exchange;
            if kind == 429 || kind == 500 { 3 } else { 1 }
        ]);
        fs::write(
            target
                .join("nci")
                .join(entry.file_name().ok_or("filename")?),
            serde_json::to_vec(&record).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let out = invoke(env, &globals(search, "--replay", &target), 4)?;
        if out.pointer("/sources/0/reason") != Some(&json!(reason)) || out.get("page").is_some() {
            return Err(format!(
                "NCI failure {kind} asserted a count or wrong failure"
            ));
        }
    }
    Ok(())
}
