//! Synthetic source and identity faults never establish empty biology or poison the cache.
use super::{
    Environment, Stub, Value, compared, fixture, fs, invoke, json, parity, path,
    recorded_stub, setup,
};

pub(super) fn run(measure: bool) -> Result<(), String> {
    for surface in ["search", "card"] {
        for fault in ["429", "500", "broken", "html", "timeout"] {
            failure(surface, fault, measure)?;
        }
    }
    for fault in ["accession", "xref", "bad-version", "zero-version"] {
        no_admission(fault, measure)?;
    }
    provenance(measure)?;
    ambiguous(measure)?;
    disabled(measure)?;
    usage()?;
    Ok(())
}
fn words(surface: &str) -> Vec<&str> {
    if surface == "search" {
        vec!["search", "cell-line", "MOLM13"]
    } else {
        vec!["get", "cell-line", "CVCL_2119"]
    }
}
fn failure(surface: &str, fault: &str, measure: bool) -> Result<(), String> {
    let stub = Stub::for_source("api.cellosaurus.org", "application/json", |_| {
        Ok("{}".into())
    })?;
    let env = setup(&stub)?;
    let directory = env.root().join("seed");
    let words = words(surface);
    let mut args = words.clone();
    args.extend(["--record", directory.to_str().ok_or("path")?]);
    invoke(&env, &args, 4)?;
    if stub.finish()?.len() != 1 {
        return Err("fault seed fanout".into());
    }
    let body = if fault == "broken" {
        "{"
    } else if fault == "html" {
        "<html>verification</html>"
    } else {
        "{}"
    };
    for entry in fs::read_dir(directory.join("cellosaurus")).map_err(|e| e.to_string())? {
        let file = entry.map_err(|e| e.to_string())?.path();
        let mut record: Value =
            serde_json::from_slice(&fs::read(&file).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let exchange = json!(
            {"status":fault.parse::<u16>().unwrap_or(200),"content_type":if fault=="html"{"text/html"}else{"application/json"},
            "delay_ms":if fault=="timeout"{30000}else{0},"retry_after_ms":null,"body":{"kind":"text","value":body}}
        );
        *record.get_mut("exchanges").ok_or("exchanges")? =
            json!([exchange.clone(), exchange.clone(), exchange]);
        fs::write(file, record.to_string()).map_err(|e| e.to_string())?;
    }
    let mut replay = words.clone();
    replay.extend(["--replay", directory.to_str().ok_or("path")?]);
    let value = invoke(&env, &replay, 4)?;
    failed(&value)?;
    compared(
        &format!("synthetic/{surface}/{fault}"),
        &env,
        &words,
        4,
        measure,
        &directory,
        value,
    )?;
    Ok(())
}
fn failed(value: &Value) -> Result<(), String> {
    path(value, "/status", json!("failed"))?;
    path(value, "/sources/0/source", json!("cellosaurus"))?;
    path(value, "/sources/0/state", json!("failed"))?;
    path(value, "/sources/0/use", json!("open"))?;
    if value
        .pointer("/sources/0/terms_url")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        return Err("failure lost terms".into());
    }
    if value.pointer("/result/header").is_some() || value.pointer("/result/items").is_some() {
        return Err("failed reply became identity or empty result".into());
    }
    Ok(())
}
fn no_admission(fault: &'static str, measure: bool) -> Result<(), String> {
    let (bad, healthy) = fault_card(fault)?;
    let release = fixture("portable-cellosaurus-cellosaurus-release.json")?;
    let reverse = fixture("portable-cellosaurus-search-reverse.json")?;
    let mut served = 0;
    let stub = Stub::for_source_status(
        "api.cellosaurus.org",
        "application/json",
        move |request| {
            if request.contains("/release-info?") {
                return Ok(release.clone());
            }
            if request.contains("/search/cell-line?") {
                return Ok(reverse.clone());
            }
            served += 1;
            if served <= 2 {
                Ok(bad.clone())
            } else {
                Ok(healthy.clone())
            }
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let words = if fault == "xref" {
        vec!["get", "cell-line", "DepMap:ACH-000362"]
    } else {
        words("card")
    };
    let optional = matches!(fault, "bad-version" | "zero-version");
    let exit = if optional { 0 } else { 4 };
    // The first ordinary read exercises cache admission before recording bypasses it.
    let initial = invoke(&env, &words, exit)?;
    if optional {
        omitted_version(&initial)?;
    } else {
        failed(&initial)?;
    }
    let failed_value = parity(
        &format!("synthetic/no-admission/{fault}/failed"),
        &env,
        &words,
        exit,
        measure,
    )?;
    if optional {
        omitted_version(&failed_value)?;
    } else {
        failed(&failed_value)?;
    }
    // Ordinary live read must fetch a healthy value instead of reusing a rejected cache entry.
    let ordinary = invoke(&env, &words, 0)?;
    if optional {
        omitted_version(&ordinary)?;
    }
    let repaired = parity(
        &format!("synthetic/no-admission/{fault}/healthy"),
        &env,
        &words,
        0,
        measure,
    )?;
    path(&repaired, "/result/header/accession", json!("CVCL_2119"))?;
    if optional {
        path(&repaired, "/result/dates/version", json!(48))?;
    }
    let calls = stub.finish()?;
    if calls
        .iter()
        .filter(|(_, r)| r.contains("GET /cell-line/"))
        .count()
        != if optional { 3 } else { 4 }
    {
        return Err(format!(
            "{fault}: identity rejection or optional-field cache admission changed"
        ));
    }
    Ok(())
}
fn provenance(measure: bool) -> Result<(), String> {
    let card = fixture("native-projected-molm13.json")?;
    let stub = Stub::for_source_status(
        "api.cellosaurus.org",
        "application/json",
        move |request| {
            if request.contains("/release-info?") {
                Ok("{}".into())
            } else {
                Ok(card.clone())
            }
        },
        |r| {
            if r.contains("/release-info?") {
                500
            } else {
                200
            }
        },
    )?;
    let env = setup(&stub)?;
    let value = parity(
        "synthetic/card/release500",
        &env,
        &words("card"),
        0,
        measure,
    )?;
    path(&value, "/status", json!("partial"))?;
    path(&value, "/problems/0/field", json!("release"))?;
    if value.pointer("/result/release").is_some() || value.pointer("/result/fetched").is_none() {
        return Err("failed provenance fabricated release or hid retrieval date".into());
    }
    stub.finish()?;
    Ok(())
}
fn ambiguous(measure: bool) -> Result<(), String> {
    let candidates = fixture("native-ambiguous-reverse.json")?;
    let release = fixture("portable-cellosaurus-cellosaurus-release.json")?;
    let stub = Stub::for_source_status(
        "api.cellosaurus.org",
        "application/json",
        move |request| {
            if request.contains("/release-info?") {
                Ok(release.clone())
            } else if request.contains("/search/cell-line?") {
                Ok(candidates.clone())
            } else {
                Err("ambiguous reverse fetched a selected card".into())
            }
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let value = parity(
        "synthetic/reverse/ambiguous-DepMap",
        &env,
        &["get", "cell-line", "DepMap:ACH-000362"],
        0,
        measure,
    )?;
    path(&value, "/status", json!("partial"))?;
    if value.pointer("/result/header").is_some() {
        return Err("ambiguous search chose identity".into());
    }
    let rows = value
        .pointer("/result/candidates")
        .and_then(Value::as_array)
        .ok_or("candidates")?;
    let next = value.get("next").and_then(Value::as_array).ok_or("next")?;
    if rows.len() != 5 || next.len() != 5 {
        return Err("ambiguous candidates or actions lost".into());
    }
    stub.finish()?;
    Ok(())
}
fn disabled(measure: bool) -> Result<(), String> {
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    invoke(&env, &words("card"), 0)?;
    let disabled_words = [
        "get",
        "cell-line",
        "CVCL_2119",
        "--disable-sources",
        "cellosaurus",
    ];
    let disabled = invoke(&env, &disabled_words, 4)?;
    path(&disabled, "/sources/0/state", json!("skipped"))?;
    if disabled.pointer("/result/header").is_some() {
        return Err("disabled source returned warm identity".into());
    }
    parity(
        "synthetic/disabled/prime-card",
        &env,
        &words("card"),
        0,
        measure,
    )?;
    let value = parity(
        "synthetic/disabled/warm-card",
        &env,
        &[
            "get",
            "cell-line",
            "CVCL_2119",
            "--disable-sources",
            "cellosaurus",
        ],
        4,
        measure,
    )?;
    path(&value, "/sources/0/state", json!("skipped"))?;
    if stub.finish()?.len() != 4 {
        return Err("disabled source used cache or fetched".into());
    }
    Ok(())
}
fn usage() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    for [option, value] in [["--limit", "0"], ["--limit", "26"], ["--offset", "-1"]] {
        let words = ["search", "cell-line", "MOLM13", option, value];
        let value = invoke(&env, &words, 2)?;
        path(&value, "/status", json!("usage"))?;
    }
    let value = invoke(&env, &["get", "cell-line", "ArrayExpress:E-MTAB-2770"], 2)?;
    path(&value, "/status", json!("usage"))?;
    Ok(())
}

fn omitted_version(value: &Value) -> Result<(), String> {
    path(value, "/status", json!("partial"))?;
    path(value, "/result/header/accession", json!("CVCL_2119"))?;
    if value.pointer("/result/dates/version").is_some()
        || !value
            .get("problems")
            .and_then(Value::as_array)
            .is_some_and(|rows| {
                rows.iter()
                    .any(|p| p.get("field") == Some(&json!("entry-version")))
            })
    {
        return Err(format!(
            "malformed optional version lost identity or precise omission: {value}"
        ));
    }
    Ok(())
}

fn fault_card(fault: &str) -> Result<(String, String), String> {
    let mut valid: Value =
        serde_json::from_str(&fixture("portable-cellosaurus-cell-CVCL_2119.json")?)
            .map_err(|e| e.to_string())?;
    let healthy = valid.to_string();
    let row = valid
        .pointer_mut("/Cellosaurus/cell-line-list/0")
        .ok_or("record")?;
    match fault {
        "accession" => {
            *row.pointer_mut("/accession-list/0/value")
                .ok_or("accession")? = json!("CVCL_0004")
        }
        "xref" => {
            row.as_object_mut().ok_or("record")?.remove("xref-list");
        }
        "bad-version" => *row.get_mut("entry-version").ok_or("version")? = json!("bad"),
        _ => *row.get_mut("entry-version").ok_or("version")? = json!("0"),
    }
    let bad = valid.to_string();
    Ok((bad, healthy))
}
