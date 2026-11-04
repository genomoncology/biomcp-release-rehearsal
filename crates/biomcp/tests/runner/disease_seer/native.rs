//! Explicit retained-source command proof; native bodies remain outside Git.
use super::{args, invoke, markdown, measure, record, replay, setup};
use crate::runner::launch::native_budget as budget;
use crate::runner::network_stub::Stub;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
pub(in crate::runner) fn run(root: &Path) -> Result<(), String> {
    let plan: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("plan.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let mut replies = BTreeMap::new();
    for row in &plan {
        let body = fs::read(root.join(text(row, "body")?)).map_err(|e| e.to_string())?;
        let hash = Sha256::digest(&body)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if hash != text(row, "sha256")? {
            return Err("retained SEER body hash differs".into());
        }
        replies.insert(
            target(text(row, "url")?),
            String::from_utf8(body).map_err(|e| e.to_string())?,
        );
    }
    let count = plan.len();
    let stub = Stub::for_source_status(
        "seer.cancer.gov",
        "application/json",
        move |request| {
            let path = request
                .lines()
                .next()
                .and_then(|r| r.split_whitespace().nth(1))
                .ok_or("missing target")?;
            replies
                .get(path)
                .cloned()
                .ok_or_else(|| format!("unplanned SEER request {path}"))
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let output = root.join("product-proof");
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    for row in &plan {
        let name = text(row, "name")?;
        let directory = output.join(format!("record-{name}"));
        let words = commands(name)?;
        let refs = words.iter().map(String::as_str).collect::<Vec<_>>();
        record(&env, &refs, &directory, 0)?;
        restore_date(
            &directory,
            row.get("fetched")
                .and_then(Value::as_u64)
                .ok_or("missing original fetch")?,
        )?;
        let mut replay_words = words.clone();
        replay_words.extend(["--replay".into(), directory.to_string_lossy().into()]);
        let result = invoke(&env, &replay_words, 0)?;
        measure(&refs, &result)?;
        if result.pointer("/result/fetched") != row.get("fetched") {
            return Err("original fetch time changed".into());
        }
        let last = replay(&directory, steps(name, &words)?)?;
        verify_last(name, &last)?;
        markdown(
            &env,
            &refs,
            &directory,
            &["## Result", "## Sources", "SEER/NCI"],
        )?;
        fs::write(
            output.join(format!("{name}.json")),
            serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        measure_pages(&env, name, &words, &directory, &output)?;
    }
    if stub.finish()?.len() != count {
        return Err("native command proof changed source delivery count".into());
    }
    Ok(())
}
fn verify_last(name: &str, value: &Value) -> Result<(), String> {
    if name == "survival"
        && value
            .pointer("/result/items")
            .and_then(Value::as_array)
            .is_none_or(|rows| rows.iter().skip(1).any(|row| row.get("observed").is_some()))
    {
        return Err("native model-only years acquired observed estimates".into());
    }
    if name == "catalog" && value.get("next").is_some() {
        return Err("terminal native catalog page has another page action".into());
    }
    Ok(())
}
fn measure_pages(
    env: &super::Environment,
    name: &str,
    words: &[String],
    directory: &Path,
    output: &Path,
) -> Result<(), String> {
    let (total, option) = match name {
        "catalog" => (73, "--offset"),
        "incidence" => (5, "--section-offset"),
        _ => (48, "--section-offset"),
    };
    let mut rows = Vec::new();
    for offset in (0..total).step_by(5) {
        let mut command = words.to_vec();
        command.extend([
            option.into(),
            offset.to_string(),
            "--replay".into(),
            directory.to_string_lossy().into(),
        ]);
        let value = invoke(env, &command, 0)?;
        let full = serde_json::to_vec(&value).map_err(|e| e.to_string())?.len();
        let comparable = crate::runner::source_terms::size(&value, "")?;
        rows.push(json!({"offset":offset,"full":full,"comparable":comparable}));
        fs::write(
            output.join(format!("{name}-{offset}.json")),
            serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    fs::write(
        output.join(format!("measurements-{name}.json")),
        serde_json::to_vec_pretty(&rows).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing {field}"))
}
fn target(url: &str) -> String {
    url.strip_prefix("https://seer.cancer.gov")
        .unwrap_or(url)
        .into()
}
fn commands(name: &str) -> Result<Vec<String>, String> {
    match name {
        "catalog" => Ok(args(&["search", "disease", "--source", "seer"])),
        "incidence" => Ok(args(&[
            "get",
            "disease",
            "47",
            "incidence",
            "--source",
            "seer",
            "--sex",
            "1",
            "--race",
            "1",
            "--age-range",
            "1",
            "--dataset",
            "4",
            "--period",
            "5",
        ])),
        "survival" => Ok(args(&[
            "get",
            "disease",
            "55",
            "survival",
            "--source",
            "seer",
            "--sex",
            "3",
            "--race",
            "1",
            "--age-range",
            "1",
            "--dataset",
            "4",
            "--interval",
            "5",
        ])),
        _ => Err("unknown native witness".into()),
    }
}
fn steps(name: &str, words: &[String]) -> Result<Value, String> {
    let mut steps = vec![json!(
        {"run":words,"mcp":words,"expect":{"/sources/0/source":"seer","/sources/0/state":"ok"},"max_bytes":budget(&format!("seer/{name}"))?}
    )];
    match name {
        "catalog" => {
            steps.push(json!(
                {"run":words,"offline":true,"expect":{"/page/total":73,"/result/items/0/code":"1",
                "/result/items/0/label":"All Cancer Sites Combined","/result/items/0/active":true},"max_bytes":budget("seer/catalog")?}
            ));
            steps.push(json!(
                {"run":["search","disease","--source","seer","--offset","12"],"mcp":["search","disease","--source","seer","--offset","12"],
                "offline":true,"expect":{"/result/items/0/code":"55","/result/items/0/label":"Breast","/page/total":73},"max_bytes":budget("seer/catalog")?}
            ));
            steps.push(json!(
                {"run":["search","disease","--source","seer","--offset","70"],"offline":true,"expect":{"/page/returned":3,"/page/total":73},
                "max_bytes":budget("seer/catalog-tail")?}
            ));
        }
        "incidence" => {
            steps.push(json!(
                {"run":words,"offline":true,"expect":{"/page/total":5,"/result/items/0/observed/value/value":47.172044,
                "/result/items/0/observed/count":434448,"/result/items/0/returned_selectors/3/code":"101",
                "/result/items/4/returned_selectors/3/code":"107"},"max_bytes":budget("seer/incidence")?}
            ));
            let mut middle = words.to_vec();
            middle.extend([
                "--section-limit".into(),
                "1".into(),
                "--section-offset".into(),
                "1".into(),
            ]);
            steps.push(json!(
                {"run":middle,"mcp":middle,"offline":true,"expect":{"/page/returned":1,"/page/total":5,
                "/result/items/0/returned_selectors/3/code":"104","/result/items/0/observed/value/value":13.624780,
                "/result/items/0/observed/count":125024},"max_bytes":budget("seer/incidence")?}
            ));
            let mut terminal = words.to_vec();
            terminal.extend(["--section-offset".into(), "5".into()]);
            steps.push(json!({"run":terminal,"offline":true,"expect":{"/page/returned":0,"/page/total":5},"max_bytes":budget("seer/incidence-terminal")?}));
        }
        "survival" => {
            let mut last = words.to_vec();
            last.extend(["--section-offset".into(), "43".into()]);
            steps.push(json!(
                {"run":last,"mcp":last,"offline":true,"expect":{"/page/total":48,"/result/items/0/year":2018,
                "/result/items/0/observed/value/value":93.862708,"/result/items/0/modeled/value/value":93.171232,"/result/items/4/year":2022,
                "/result/items/4/modeled/value/value":93.690345,"/result/items/4/returned_selectors/5/code":"31"},"max_bytes":budget("seer/survival-tail")?}
            ));
        }
        _ => return Err("unknown native witness".into()),
    }
    Ok(Value::Array(steps))
}
fn restore_date(directory: &Path, fetched: u64) -> Result<(), String> {
    for file in fs::read_dir(directory.join("seer")).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let mut row: Value =
            serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        for exchange in row
            .get_mut("exchanges")
            .and_then(Value::as_array_mut)
            .ok_or("missing exchanges")?
        {
            exchange
                .as_object_mut()
                .ok_or("bad exchange")?
                .insert("fetched".into(), json!(fetched));
        }
        fs::write(
            file.path(),
            serde_json::to_vec_pretty(&row).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
