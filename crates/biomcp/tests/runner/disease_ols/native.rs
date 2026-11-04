//! Retained source bytes enter only this explicit external-evidence proof.
use super::{args, markdown, record, replay, setup};
use crate::runner::launch::native_budget as budget;
use crate::runner::network_stub::Stub;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::{collections::BTreeMap, fs, path::Path};
pub(in crate::runner) fn run(root: &Path) -> Result<(), String> {
    let requests = load(root)?;
    let mut replies = BTreeMap::new();
    for row in &requests {
        replies.insert(
            key(row
                .pointer("/url")
                .unwrap_or(&Value::Null)
                .as_str()
                .ok_or("missing url")?),
            fs::read_to_string(
                root.join(
                    row.pointer("/local_body")
                        .unwrap_or(&Value::Null)
                        .as_str()
                        .ok_or("missing body")?,
                ),
            )
            .map_err(|e| e.to_string())?,
        );
    }
    let stub = Stub::for_source_status(
        "www.ebi.ac.uk",
        "application/json",
        move |request| {
            let path = request
                .lines()
                .next()
                .and_then(|s| s.split_whitespace().nth(1))
                .ok_or("missing target")?;
            replies
                .get(&key(path))
                .cloned()
                .ok_or_else(|| format!("unplanned source request: {path}"))
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let output = root.join("product-proof");
    fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    for (index, row) in requests.iter().enumerate() {
        prove_row(root, &output, &env, index, row)?;
    }
    if stub.finish()?.len() != requests.len() {
        return Err("unexpected source attempt count".into());
    }
    Ok(())
}
fn prove_row(
    root: &Path,
    output: &Path,
    env: &super::Environment,
    index: usize,
    row: &Value,
) -> Result<(), String> {
    let words = command(row)?;
    let directory = output.join(format!("record-{index}"));
    let refs = words.iter().map(String::as_str).collect::<Vec<_>>();
    let result = record(env, &refs, &directory)?;
    restore_date(
        &directory,
        row.pointer("/fetched")
            .unwrap_or(&Value::Null)
            .as_u64()
            .ok_or("missing original date")?,
    )?;
    let body: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join(
                row.pointer("/local_body")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .ok_or("missing body")?,
            ),
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let expected = expectations(&body)?;

    let size = serde_json::to_vec(&result)
        .map_err(|e| e.to_string())?
        .len();
    let mut steps =
        vec![json!({"run":words,"mcp":words,"expect":expected,"max_bytes":page_cap(index)?})];
    section_steps(row, &mut steps)?;
    replay(&directory, Value::Array(steps))?;
    markdown(env, &refs, &directory, &["## Result", "## Sources"])?;
    if row
        .pointer("/url")
        .and_then(Value::as_str)
        .is_some_and(|s| s.contains("DOID%3A3910"))
    {
        section_outputs(env, &directory, output)?;
    }
    fs::write(
        output.join(format!("output-{index}.json")),
        serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    writeln!(
        std::io::stderr(),
        "native disease proof {index}: {size} full JSON bytes; original {}",
        row.pointer("/sha256").unwrap_or(&Value::Null)
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn page_cap(index: usize) -> Result<usize, String> {
    // Ceil(j11984 comparable CLI bytes * 1.15), exact validated service terms excluded.
    budget(&format!("ols/page/{index}"))
}
fn section_outputs(
    env: &super::Environment,
    directory: &Path,
    output: &Path,
) -> Result<(), String> {
    for section in ["synonyms", "xrefs"] {
        let words = [
            "get",
            "disease",
            "DOID:3910",
            section,
            "--source",
            "ols",
            "--ontology",
            "doid",
        ];
        let mut command = args(&words);
        command.extend(["--replay".into(), directory.to_string_lossy().into()]);
        let result = super::invoke(env, &command, 0)?;
        let full = serde_json::to_vec(&result)
            .map_err(|e| e.to_string())?
            .len();
        let comparable = crate::runner::source_terms::size(&result, "")?;
        writeln!(
            std::io::stderr(),
            "native {section}: full={full}, comparable={comparable}"
        )
        .map_err(|e| e.to_string())?;
        if section == "synonyms"
            && result
                .pointer("/result/items")
                .and_then(Value::as_array)
                .is_none_or(|rows| rows.iter().any(|r| r.get("scope").is_some()))
        {
            return Err("DOID missing scopes were invented".into());
        }
        fs::write(
            output.join(format!("{section}.json")),
            serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        replay(
            directory,
            json!([{"run":words,"mcp":words,"expect":{"/page/offset":0},"max_bytes":budget(&format!("ols/section/{section}"))?}]),
        )?;
    }
    Ok(())
}

fn section_steps(row: &Value, steps: &mut Vec<Value>) -> Result<(), String> {
    if row
        .pointer("/url")
        .unwrap_or(&Value::Null)
        .as_str()
        .is_some_and(|s| s.contains("DOID%3A3910"))
    {
        for section in ["synonyms", "xrefs"] {
            for offset in [0, 2] {
                let q = vec![
                    "get".to_owned(),
                    "disease".into(),
                    "DOID:3910".into(),
                    section.into(),
                    "--source".into(),
                    "ols".into(),
                    "--ontology".into(),
                    "doid".into(),
                    "--section-limit".into(),
                    "2".into(),
                    "--section-offset".into(),
                    offset.to_string(),
                ];
                steps.push(json!(
                    {"run":q,"mcp":q,"expect":{"/page/offset":offset},"max_bytes":budget(&format!("ols/section-page/{section}/{offset}"))?}
                ));
            }
        }
    }
    Ok(())
}

fn expectations(body: &Value) -> Result<serde_json::Map<String, Value>, String> {
    let mut expected = serde_json::Map::new();

    expected.insert("/status".into(), json!("complete"));
    if let Some(response) = body.get("response") {
        expected.insert(
            "/page/total".into(),
            response
                .pointer("/numFound")
                .unwrap_or(&Value::Null)
                .clone(),
        );
        expected.insert(
            "/page/offset".into(),
            response.pointer("/start").unwrap_or(&Value::Null).clone(),
        );
        let count = response
            .get("docs")
            .and_then(Value::as_array)
            .ok_or("missing source rows")?
            .len();
        let offset = response
            .get("start")
            .and_then(Value::as_u64)
            .ok_or("missing source offset")?;
        let total = response
            .get("numFound")
            .and_then(Value::as_u64)
            .ok_or("missing source total")?;
        if offset + (count as u64) < total {
            expected.insert(format!("/next/{count}/kind"), json!("page"));
        } else {
            expected.insert("/next".into(), json!({"count":count}));
        }
        for (i, hit) in response
            .pointer("/docs")
            .unwrap_or(&Value::Null)
            .as_array()
            .ok_or("missing docs")?
            .iter()
            .enumerate()
        {
            expected.insert(
                format!("/result/items/{i}/concept/id"),
                hit.pointer("/obo_id").unwrap_or(&Value::Null).clone(),
            );
        }
    } else {
        let term = &body.pointer("/_embedded/terms/0").unwrap_or(&Value::Null);
        expected.insert(
            "/result/returned_id".into(),
            term.pointer("/obo_id").unwrap_or(&Value::Null).clone(),
        );
        expected.insert(
            "/result/obsolete".into(),
            term.pointer("/is_obsolete").unwrap_or(&Value::Null).clone(),
        );
        if term.pointer("/is_obsolete").unwrap_or(&Value::Null) == true {
            expected.insert("/result".into(), json!({"omits":["replaced_by"]}));
            expected.insert(
                "/result/consider".into(),
                json!(["MONDO:0800444", "MONDO:0800445"]),
            );
        }
    }
    Ok(expected)
}

fn load(root: &Path) -> Result<Vec<Value>, String> {
    let rows: Vec<Value> = serde_json::from_slice(
        &fs::read(root.join("native-inputs.json")).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    for row in &rows {
        let bytes = fs::read(
            root.join(
                row.pointer("/local_body")
                    .unwrap_or(&Value::Null)
                    .as_str()
                    .ok_or("missing body")?,
            ),
        )
        .map_err(|e| e.to_string())?;
        let hash = Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if row.pointer("/sha256").and_then(Value::as_str) != Some(hash.as_str()) {
            return Err("retained source hash mismatch".into());
        }
    }
    Ok(rows)
}
fn key(url: &str) -> String {
    let path = url.strip_prefix("https://www.ebi.ac.uk").unwrap_or(url);
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let mut fields = query.split('&').map(decode).collect::<Vec<_>>();
    fields.sort();
    format!("{path}?{}", fields.join("&"))
}
fn decode(value: &str) -> String {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(&byte) = value.as_bytes().get(i) {
        if byte == b'%'
            && let Some(pair) = value.get(i + 1..i + 3)
            && let Ok(decoded) = u8::from_str_radix(pair, 16)
        {
            out.push(decoded);
            i += 3;
            continue;
        }
        out.push(if byte == b'+' { b' ' } else { byte });
        i += 1;
    }
    String::from_utf8_lossy(&out).into()
}

fn command(row: &Value) -> Result<Vec<String>, String> {
    let url = row
        .pointer("/url")
        .unwrap_or(&Value::Null)
        .as_str()
        .ok_or("missing url")?;
    let mut fields = BTreeMap::new();
    for pair in url.split_once('?').ok_or("missing query")?.1.split('&') {
        let (k, v) = pair.split_once('=').ok_or("missing value")?;
        fields.insert(k, decode(v));
    }
    if let Some(text) = fields.get("q") {
        Ok(vec![
            "search".into(),
            "disease".into(),
            text.clone(),
            "--ontology".into(),
            fields.get("ontology").ok_or("missing ontology")?.clone(),
            "--limit".into(),
            "5".into(),
            "--offset".into(),
            fields.get("start").ok_or("missing offset")?.clone(),
        ])
    } else {
        let ontology = url
            .split("/ontologies/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .ok_or("missing ontology")?;
        let mut q = args(&[
            "get",
            "disease",
            fields.get("obo_id").ok_or("missing id")?,
            "--source",
            "ols",
            "--ontology",
            ontology,
        ]);
        q.shrink_to_fit();
        Ok(q)
    }
}
fn restore_date(root: &Path, fetched: u64) -> Result<(), String> {
    for file in fs::read_dir(root.join("ols")).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        if file.path().extension().is_some_and(|x| x == "json") {
            let mut value: Value =
                serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            for exchange in value
                .get_mut("exchanges")
                .ok_or("missing exchanges")?
                .as_array_mut()
                .ok_or("missing exchanges")?
            {
                *exchange.get_mut("fetched").ok_or("missing fetch date")? = json!(fetched);
            }
            fs::write(
                file.path(),
                serde_json::to_vec(&value).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
