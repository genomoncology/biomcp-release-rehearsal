//! Ordinary execution proves checked cache admission and policy before replay parity.
use super::{
    Stub, Value, Write, budget, expected, fixture, invoke, json, parity, path, recorded_stub, setup,
};
pub(super) fn run(measure: bool) -> Result<(), String> {
    for fault in ["xref", "provenance"] {
        admission(fault, measure)?;
    }
    disabled(measure)?;
    ambiguous(measure)?;
    release_failure(measure)?;
    identity(measure)
}
fn admission(fault: &str, measure: bool) -> Result<(), String> {
    let native = fixture("portable-cellosaurus-cell-CVCL_2119.json")?;
    let mut bad: Value = serde_json::from_str(&native).map_err(|e| e.to_string())?;
    let pointer = if fault == "xref" {
        "/Cellosaurus/cell-line-list/0/xref-list/0/accession"
    } else {
        "/Cellosaurus/cell-line-list/0/sequence-variation-list/0/source-list/0/reference/resource-internal-ref"
    };
    *bad.pointer_mut(pointer).ok_or("fault field")? = json!([]);
    let bad = bad.to_string();
    let release = fixture("portable-cellosaurus-cellosaurus-release.json")?;
    let mut reads = 0;
    let stub = Stub::for_source_status(
        "api.cellosaurus.org",
        "application/json",
        move |request| {
            if request.contains("/release-info?") {
                return Ok(release.clone());
            }
            reads += 1;
            Ok(if reads == 1 {
                bad.clone()
            } else {
                native.clone()
            })
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    for (phase, fresh) in [("failed", false), ("failed", false), ("healthy", true)] {
        let mut words = vec!["get", "cell-line", "CVCL_2119", "variants"];
        if fresh {
            words.push("--fresh");
        }
        let value = invoke(&env, &words, 0)?;
        let key = format!("sections/admission/{fault}/{phase}");
        ordinary(&key, &value, measure)?;
        path(
            &value,
            "/status",
            json!(if fresh { "complete" } else { "partial" }),
        )?;
        path(&value, "/result/accession", json!("CVCL_2119"))?;
        optional_variants(&value, fault, fresh)?;
    }
    let requests = stub.finish()?;
    if requests
        .iter()
        .filter(|(_, r)| r.contains("/cell-line/"))
        .count()
        != 2
    {
        return Err("optional metadata changed ordinary cache reuse or fresh repair".into());
    }
    Ok(())
}
fn optional_variants(value: &Value, fault: &str, fresh: bool) -> Result<(), String> {
    if value
        .pointer("/result/items/items")
        .and_then(Value::as_array)
        .is_none_or(|rows| rows.len() != 2)
    {
        return Err(format!(
            "optional nested field removed native variant rows: {value}"
        ));
    }
    if fault == "provenance"
        && value
            .pointer("/result/items/items/0/provenance/0/reference")
            .is_some()
            != fresh
    {
        return Err(format!(
            "optional provenance was not omitted or restored after fresh repair: {value}"
        ));
    }
    let field = if fault == "xref" {
        "xref-list"
    } else {
        "sequence-variation-list.source-list.reference"
    };
    let problem = value
        .get("problems")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter().any(|p| {
                p.get("field")
                    .and_then(Value::as_str)
                    .is_some_and(|f| f.contains(field))
            })
        });
    if problem == fresh {
        return Err(format!(
            "optional nested field changed precise refusal or fresh repair: {value}"
        ));
    }
    Ok(())
}

fn disabled(measure: bool) -> Result<(), String> {
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    let words = ["get", "cell-line", "CVCL_2119", "variants"];
    let prime = invoke(&env, &words, 0)?;
    ordinary("sections/disabled/prime", &prime, measure)?;
    let mut disabled = words.to_vec();
    disabled.extend(["--disable-sources", "cellosaurus"]);
    let value = invoke(&env, &disabled, 4)?;
    ordinary("sections/disabled/warm", &value, measure)?;
    path(&value, "/sources/0/state", json!("skipped"))?;
    if value.get("result").is_some() {
        return Err("disabled section returned cached rows".into());
    }
    if stub.finish()?.len() != 2 {
        return Err("disabled section made a request".into());
    }
    Ok(())
}
fn identity(measure: bool) -> Result<(), String> {
    let mut native: Value =
        serde_json::from_str(&fixture("portable-cellosaurus-cell-CVCL_2119.json")?)
            .map_err(|e| e.to_string())?;
    let warnings: Vec<_> = (0..5)
        .map(|i| json!({"category":"Caution","value":format!("Stock {i}: original punctuation; full qualified warning.")}))
        .collect();
    native
        .pointer_mut("/Cellosaurus/cell-line-list/0")
        .and_then(Value::as_object_mut)
        .ok_or("row")?
        .insert("comment-list".into(), json!(warnings));
    let body = native.to_string();
    let release = fixture("portable-cellosaurus-cellosaurus-release.json")?;
    let stub = Stub::for_source_status(
        "api.cellosaurus.org",
        "application/json",
        move |request| {
            Ok(if request.contains("/release-info?") {
                release.clone()
            } else {
                body.clone()
            })
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let value = parity(
        "sections/synthetic/qualified-identity",
        &env,
        &[
            "get",
            "cell-line",
            "CVCL_2119",
            "identity",
            "--section-offset",
            "3",
        ],
        0,
        measure,
    )?;
    expected::page(
        native
            .pointer("/Cellosaurus/cell-line-list/0")
            .ok_or("source")?,
        "identity",
        &value,
        3,
        5,
    )?;
    stub.finish()?;
    Ok(())
}
pub(super) fn ordinary(phase: &str, value: &Value, measure: bool) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len() + 1;
    writeln!(std::io::stderr(), "CELL_LINE_MEASURE {phase} [cli] {bytes}")
        .map_err(|e| e.to_string())?;
    if !measure {
        budget(phase, "cli", value)?;
    }
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
                Err("ambiguous section fetched a selected card".into())
            }
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let value = parity(
        "sections/synthetic/ambiguous-variants",
        &env,
        &["get", "cell-line", "DepMap:ACH-000362", "variants"],
        0,
        measure,
    )?;
    path(&value, "/status", json!("partial"))?;
    let rows = value
        .pointer("/result/candidates")
        .and_then(Value::as_array)
        .ok_or("candidates")?;
    let next = value.get("next").and_then(Value::as_array).ok_or("next")?;
    if rows.len() != 5 || next.len() != 5 || value.pointer("/result/header").is_some() {
        return Err("ambiguous section lost candidates or chose a card".into());
    }
    for (row, action) in rows.iter().zip(next) {
        path(
            action,
            "/args",
            json!(["get", "cell-line", row.get("accession").ok_or("accession")?]),
        )?;
    }
    if stub.finish()?.len() != 2 {
        return Err("ambiguous section requested a card".into());
    }
    Ok(())
}

pub(super) fn release_failure(measure: bool) -> Result<(), String> {
    let card = fixture("portable-cellosaurus-cell-CVCL_2119.json")?;
    let stub = Stub::for_source_status(
        "api.cellosaurus.org",
        "application/json",
        move |request| {
            Ok(if request.contains("/release-info?") {
                "{}".into()
            } else {
                card.clone()
            })
        },
        |request| {
            if request.contains("/release-info?") {
                500
            } else {
                200
            }
        },
    )?;
    let env = setup(&stub)?;
    let value = parity(
        "sections/synthetic/release-failure",
        &env,
        &["get", "cell-line", "CVCL_2119", "variants"],
        0,
        measure,
    )?;
    path(&value, "/status", json!("partial"))?;
    path(&value, "/problems/0/field", json!("release"))?;
    if value.pointer("/result/release").is_some()
        || value
            .pointer("/result/fetched")
            .and_then(Value::as_u64)
            .is_none()
    {
        return Err("section release failure fabricated release or lost retrieval time".into());
    }
    stub.finish()?;
    Ok(())
}
