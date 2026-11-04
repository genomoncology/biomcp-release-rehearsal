//! Native positives remain outside Git; actual record/replay exercises both public callers.
use super::{Stub, invoke, setup};
use crate::runner::launch::native_budget as budget;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

pub(in crate::runner) fn run(root: &Path) -> Result<(), String> {
    let output = root.join("0137-command-proof");
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    let replies = replies(root, &output)?;
    let stub = Stub::for_sources(
        &[
            "mydisease.info",
            "clinicaltrials.gov",
            "clinicaltrialsapi.cancer.gov",
        ],
        |_| "application/json",
        move |request| {
            let target = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .ok_or("missing request target")?;
            replies
                .get(target)
                .cloned()
                .ok_or_else(|| format!("undeclared native request: {target}"))
        },
    )?;
    let env = setup(&stub)?;
    for (name, id, source, exit, expect) in cases() {
        let words = ["search", "trial", "--condition-id", id, "--source", source];
        let mut words: Vec<String> = words.iter().map(|s| (*s).into()).collect();
        if source == "nci" {
            words.extend(["--limit".into(), "1".into()]);
        }
        let directory = env.root().join(name);
        let mut args = words.clone();
        args.extend([
            "--fresh".into(),
            "--record".into(),
            directory.to_string_lossy().into(),
        ]);
        let value = invoke(&env, &args, exit)?;
        let full = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
        fs::write(output.join(format!("{name}.json")), &full).map_err(|e| e.to_string())?;
        copy(&directory, &output.join(name))?;
        let comparable = crate::runner::source_terms::size(&value, "")?;
        let ceiling = budget(&format!("trial-condition/{name}"))?;
        let step = json!({"run":words,"mcp":words,"exit":exit,"is_error":exit != 0,"expect":expect,"max_bytes":ceiling,"max_tokens":ceiling.div_ceil(4)});
        let case = json!(
            {"feature":"trial-search","story":"Exact captured concept determines only the literal source filter","replay":output.join(name),
            "config":{"keys":{"nci_api_key":"synthetic-trial-condition-key"}},"steps":[step]}
        );
        let file = output.join(format!("{name}-case.json"));
        fs::write(
            &file,
            serde_json::to_vec_pretty(&case).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        crate::runner::run_case(&file, crate::runner::schema::Registry::embedded()?.as_ref())?;
        fs::write(
            output.join(format!("{name}-size.json")),
            json!(
                {"full_bytes":full.len(),"comparable_bytes":comparable,"max_bytes":ceiling,"token_estimate":comparable.div_ceil(4)}
            )
            .to_string(),
        )
        .map_err(|e| e.to_string())?;
    }
    explicit(&output)?;
    finish(stub, &output)
}

fn cases() -> Vec<(&'static str, &'static str, &'static str, i32, Value)> {
    vec![
        (
            "ctgov-primary",
            "MONDO:0005105",
            "ctgov",
            0,
            json!(
                {"/status":"complete","/query/condition_id":"MONDO:0005105","/result/condition_resolution/resolver":"mydisease",
                "/result/condition_resolution/concept/id":"MONDO:0005105","/result/condition_resolution/concept/label":"melanoma",
                "/page/total":3780}
            ),
        ),
        (
            "nci-unique",
            "MONDO:0005105",
            "nci",
            0,
            json!(
                {"/status":"complete","/result/condition_resolution/concept/label":"melanoma",
                "/result/condition_resolution/selected_ncit":"NCIT:C3224","/result/condition_resolution/ncit_candidates":{"count":2},
                "/result/request/2/name":"diseases.nci_thesaurus_concept_id","/result/request/2/value":"C3224","/page/total":1203}
            ),
        ),
        (
            "nci-alias-ambiguous",
            "DOID:3910",
            "nci",
            4,
            json!(
                {"/status":"failed","/result/condition_resolution/concept/id":"MONDO:0005061",
                "/result/condition_resolution/alias/object/id":"DOID:3910","/result/condition_resolution/ncit_candidates":{"count":3},
                "/problems/0/field":"ncit_mapping"}
            ),
        ),
        (
            "nci-missing",
            "MONDO:0800444",
            "nci",
            4,
            json!({"/status":"failed","/result/condition_resolution/concept/id":"MONDO:0800444","/problems/0/field":"ncit_mapping"}),
        ),
    ]
}
fn replies(root: &Path, output: &Path) -> Result<BTreeMap<String, String>, String> {
    let bytes = fs::read(root.join("requests.json")).map_err(|e| e.to_string())?;
    if digest(&bytes) != "7cd1db0335a1cb96203260f4ea7887b54ebdc9e94df9f3ee8c3014f063f106ba" {
        return Err("native manifest changed".into());
    }
    let rows: Vec<Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let mut replies = BTreeMap::new();
    fs::create_dir(output.join("original-captures")).map_err(|e| e.to_string())?;
    fs::write(output.join("original-captures/requests.json"), bytes).map_err(|e| e.to_string())?;
    for row in rows.iter().filter(|row| {
        ["MONDO:0005105", "DOID:3910", "MONDO:0800444"]
            .contains(&row["requested_id"].as_str().unwrap_or(""))
    }) {
        let directory = row["directory"]
            .as_str()
            .ok_or("missing native directory")?;
        let bytes = fs::read(root.join(directory).join("body.raw")).map_err(|e| e.to_string())?;
        if digest(&bytes) != row["sha256"].as_str().ok_or("missing hash")? {
            return Err("native body changed".into());
        }
        let id = row["requested_id"]
            .as_str()
            .ok_or("missing ID")?
            .replace(':', "%3A");
        replies.insert(
            format!("/v1/disease/{id}?fields=mondo%2Cdisease_ontology"),
            String::from_utf8(bytes).map_err(|e| e.to_string())?,
        );
        copy(
            &root.join(directory),
            &output.join("original-captures").join(directory),
        )?;
    }
    for capture in ["trial/condition/ctgov", "trial/nci/search/nci"] {
        for entry in fs::read_dir(
            crate::runner::launch::fixtures()
                .join("replies")
                .join(capture),
        )
        .map_err(|e| e.to_string())?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().is_none_or(|v| v != "json") {
                continue;
            }
            let original = output.join("original-captures").join(capture);
            fs::create_dir_all(&original).map_err(|e| e.to_string())?;
            fs::copy(entry.path(), original.join(entry.file_name())).map_err(|e| e.to_string())?;
            let record: Value =
                serde_json::from_slice(&fs::read(entry.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let url = record
                .get("url")
                .and_then(Value::as_str)
                .ok_or("missing request URL")?;
            let path = url
                .split_once("://")
                .and_then(|(_, s)| s.find('/').map(|i| &s[i..]))
                .ok_or("missing request path")?;
            replies.insert(
                path.into(),
                record
                    .pointer("/exchanges/0/body/value")
                    .ok_or("missing native reply body")?
                    .to_string(),
            );
        }
    }
    Ok(replies)
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn copy(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir(to).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(from).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn explicit(output: &Path) -> Result<(), String> {
    let value = crate::runner::run_case(
        &crate::runner::launch::fixtures().join("cases/trial/nci-condition-id.yaml"),
        crate::runner::schema::Registry::embedded()?.as_ref(),
    )?;
    fs::write(
        output.join("nci-explicit.json"),
        serde_json::to_vec(&value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let comparable = crate::runner::source_terms::size(&value, "")?;
    let full = serde_json::to_vec(&value).map_err(|e| e.to_string())?.len();
    fs::write(
        output.join("nci-explicit-size.json"),
        json!(
            {"full_bytes":full,"comparable_bytes":comparable,"max_bytes":(comparable*115).div_ceil(100),"token_estimate":comparable.div_ceil(4)}
        )
        .to_string(),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn finish(stub: Stub, output: &Path) -> Result<(), String> {
    let requests = stub.finish()?;
    fs::write(
        output.join("local-requests.json"),
        serde_json::to_vec_pretty(&requests).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if requests.len() != 7 {
        return Err(format!(
            "expected seven local deliveries, got {}",
            requests.len()
        ));
    }
    use std::io::Write;
    writeln!(
        std::io::stderr(),
        "Trial condition native proof: seven local deliveries, four resolved CLI/MCP pairs and one explicit \
        NCIt pair; zero provider calls"
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
