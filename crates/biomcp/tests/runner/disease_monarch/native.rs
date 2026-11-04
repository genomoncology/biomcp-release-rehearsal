//! Explicit source command proof reads unchanged retained bodies outside Git.
use super::{Stub, invoke, measure, record, replay, setup};
use crate::runner::launch::native_budget as budget;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
pub(in crate::runner) fn run(root: &Path) -> Result<(), String> {
    let plan: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("plan.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let mut replies = BTreeMap::new();
    for row in &plan {
        let bytes = fs::read(root.join(text(row, "body")?)).map_err(|e| e.to_string())?;
        let hash = Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        if hash != text(row, "sha256")? {
            return Err("native body hash changed".into());
        }
        replies.insert(
            text(row, "target")?.to_owned(),
            String::from_utf8(bytes).map_err(|e| e.to_string())?,
        );
    }
    let stub = Stub::for_source_status(
        "api-v3.monarchinitiative.org",
        "application/json",
        move |request| {
            let target = request
                .split_whitespace()
                .nth(1)
                .ok_or("missing request target")?;
            replies
                .get(target)
                .cloned()
                .ok_or_else(|| format!("undeclared Monarch request {target}"))
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let output = root.join("product-proof");
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    for (index, row) in plan.iter().enumerate() {
        prove_row(root, &output, &env, index, row)?;
    }
    if stub.finish()?.len() != plan.len() {
        return Err("native proof request count changed".into());
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
    let id = text(row, "id")?;
    let section = text(row, "section")?;
    let offset = row
        .get("offset")
        .and_then(Value::as_u64)
        .ok_or("missing offset")?
        .to_string();
    let projected = row.get("default_limit") == Some(&json!(true));
    let mut words = vec!["get", "disease", id, section, "--source", "monarch"];
    if !projected {
        words.extend(["--section-limit", "5"]);
    }
    words.extend(["--section-offset", &offset]);
    let directory = output.join(format!("record-{index:02}"));
    let expected: Value = serde_json::from_slice(
        &fs::read(root.join(text(row, "body")?)).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let exit = if expected.get("total") == Some(&json!(0)) {
        1
    } else {
        0
    };
    let delivered = record(env, &words, &directory, exit)?;
    fs::write(
        output.join(format!("{index:02}-local-delivery.json")),
        serde_json::to_vec_pretty(&delivered).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    restore_date(
        &directory,
        row.get("fetched")
            .and_then(Value::as_u64)
            .ok_or("missing original timestamp")?,
    )?;
    let mut command = words.to_vec();
    command.extend([
        "--replay",
        directory.to_str().ok_or("nonUTF8 recording path")?,
    ]);
    let result = invoke(env, &command, exit)?;
    verify(id, &expected, &result)?;
    if result.pointer("/result/fetched") != row.get("fetched") {
        return Err("original retrieval date changed".into());
    }
    measure(&words, &result)?;
    replay(
        &directory,
        json!(
            [{"run":words,"mcp":words,"exit":exit,"expect":{"/status":if expected.get("total") == Some(&json!(0)) {"absent"} else {"complete"},
            "/page/offset":expected.get("offset").ok_or("missing offset")?,"/page/total":expected.get("total").ok_or("missing total")?,
            "/page/returned":expected.get("items").and_then(Value::as_array).ok_or("missing source items")?.len()},
            "max_bytes":if projected {default_ceiling(id,section)?} else {ceiling(id,section,&offset)?}}]
        ),
    )?;
    fs::write(
        output.join(format!("{index:02}-original-date.json")),
        serde_json::to_vec_pretty(&result).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    markdown(env, &command)?;
    Ok(())
}
fn verify(id: &str, native: &Value, result: &Value) -> Result<(), String> {
    let original = native
        .get("items")
        .and_then(Value::as_array)
        .ok_or("missing native items")?;
    let mapped = result
        .pointer("/result/items")
        .and_then(Value::as_array)
        .ok_or("missing mapped items")?;
    if original.len() != mapped.len() {
        return Err("source rows dropped or deduplicated".into());
    }
    for (raw, mapped) in original.iter().zip(mapped) {
        let association = mapped.get("association").ok_or("missing association")?;
        for (target, source) in [
            ("subject/id", "subject"),
            ("subject/label", "subject_label"),
            ("object/id", "object"),
            ("object/label", "object_label"),
            ("predicate", "predicate"),
            ("aggregator_knowledge_source", "aggregator_knowledge_source"),
            ("attribution/record", "id"),
        ] {
            if association.pointer(&format!("/{target}")) != raw.get(source) {
                return Err(format!("native {source} changed"));
            }
        }
        if association.get("requested_id") != Some(&json!(id)) {
            return Err("requested identity changed".into());
        }
        for field in [
            "original_subject",
            "original_object",
            "primary_knowledge_source",
            "provided_by",
            "negated",
            "has_count",
            "has_total",
            "has_percentage",
            "has_quotient",
            "original_predicate",
            "category",
            "agent_type",
            "knowledge_level",
            "evidence_count",
        ] {
            let expected = raw.get(field).filter(|v| !v.is_null());
            if association.get(field) != expected {
                return Err(format!("native optional {field} changed"));
            }
        }
        context(raw, association)?;
    }
    if original.is_empty() && result.get("next").is_some() {
        return Err("terminal empty page advertises a next action".into());
    }
    Ok(())
}
fn markdown(env: &super::Environment, words: &[&str]) -> Result<(), String> {
    let mut words = words.to_vec();
    words.extend(["--format", "md"]);
    let words = words.into_iter().map(String::from).collect::<Vec<_>>();
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &words)
        .output()
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    if !text.contains("## Sources") || !text.contains("Monarch source-specific licensing") {
        return Err("Markdown lost source terms".into());
    }
    Ok(())
}
fn restore_date(directory: &Path, fetched: u64) -> Result<(), String> {
    for file in fs::read_dir(directory.join("monarch")).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let mut value: Value =
            serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        for exchange in value
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
            serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value[field]
        .as_str()
        .ok_or_else(|| format!("missing {field}"))
}

fn references(
    raw: &Value,
    association: &Value,
    ids: &str,
    links: &str,
    field: &str,
) -> Result<(), String> {
    if field == "evidence"
        && raw.get(ids).is_none_or(Value::is_null)
        && raw.get(links).is_none_or(Value::is_null)
        && association.get(field).is_some()
    {
        return Err("unavailable native evidence became present".into());
    }
    let expected = raw
        .get(ids)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|id| {
            let link = raw
                .get(links)
                .and_then(Value::as_array)
                .and_then(|links| links.iter().find(|link| link.get("id") == Some(&id)));
            Ok(match link {
                Some(link) => json!({"id":id,"url":link.get("url").ok_or("missing link URL")?}),
                None => json!({"id":id}),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let actual = association
        .get(field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if actual != expected {
        return Err(format!(
            "native {field} IDs/URLs changed or invented labels"
        ));
    }
    Ok(())
}

fn context(raw: &Value, association: &Value) -> Result<(), String> {
    references(
        raw,
        association,
        "has_evidence",
        "has_evidence_links",
        "evidence",
    )?;
    references(
        raw,
        association,
        "publications",
        "publications_links",
        "publications",
    )?;
    let qualifiers = association
        .get("qualifiers")
        .and_then(Value::as_array)
        .ok_or("missing qualifiers")?;
    let expected = if raw.get("frequency_qualifier").is_none_or(Value::is_null) {
        json!([])
    } else {
        json!([{"field":"frequency_qualifier",
"id":raw.get("frequency_qualifier").ok_or("missing frequency")?,
"label":raw.get("frequency_qualifier_label").ok_or("missing frequency label")?}])
    };
    if qualifiers != expected.as_array().ok_or("bad expected qualifiers")? {
        return Err("native frequency role or label changed".into());
    }
    for field in ["subject_closure", "object_closure"] {
        let expected = raw
            .get(field)
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or(json!([]));
        if association.get(field) != Some(&expected) {
            return Err(format!("native {field} changed"));
        }
    }
    Ok(())
}

fn ceiling(id: &str, section: &str, offset: &str) -> Result<usize, String> {
    // Ceil(j12282 native comparable bytes * 1.15); all original terms remain emitted.
    budget(&format!("monarch/{id}/{section}/{offset}"))
}

fn default_ceiling(id: &str, section: &str) -> Result<usize, String> {
    // Derived-prefix local deliveries in j12430; measured comparable bytes + 15%.
    budget(&format!("monarch-default/{id}/{section}"))
}
