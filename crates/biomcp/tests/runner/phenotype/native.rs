//! Explicit proof reuses unchanged archived publisher bodies outside Git.
tool_only! { use super::{Stub, raw_invoke, setup}; }
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
tool_only! {
pub(in crate::runner) fn run(root: &Path) -> Result<(), String> {
    let plan: Vec<Value> =
        serde_json::from_slice(&fs::read(root.join("plan.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let output = root.join("product-proof");
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    for (index, row) in plan.iter().enumerate() {
        prove(root, &output, index, row)?;
    }
    Ok(())
}
}
fn text<'a>(row: &'a Value, key: &str) -> Result<&'a str, String> {
    row.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing {key}"))
}
pub(super) fn archived_body(root: &Path, row: &Value) -> Result<(Value, String), String> {
    let bytes = fs::read(root.join(text(row, "body")?)).map_err(|e| e.to_string())?;
    let hash = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    if hash != text(row, "sha256")? {
        return Err("retained body hash changed".into());
    }
    let native: Value = if row.get("status") == Some(&json!(404)) {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?
    };
    let body = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    Ok((native, body))
}

tool_only! {
fn prove(root: &Path, output: &Path, index: usize, row: &Value) -> Result<(), String> {
    let (native, body) = archived_body(root, row)?;
    let target = text(row, "target")?.to_string();
    let host = match text(row, "source")? {
        "hpo" => "ontology.jax.org",
        "monarch" => "api-v3.monarchinitiative.org",
        _ => return Err("unknown source".into()),
    };
    let status = row.get("status").and_then(Value::as_u64).unwrap_or(200) as u16;
    let stub = Stub::for_source_status(
        host,
        "application/json",
        move |request| {
            if request.split_whitespace().nth(1) != Some(&target) {
                return Err(format!("unproved request: {request}; expected {target}"));
            }
            Ok(body.clone())
        },
        move |_| status,
    )?;
    let env = setup(&stub)?;
    let words = row
        .get("words")
        .and_then(Value::as_array)
        .ok_or("words")?
        .iter()
        .map(|v| v.as_str().ok_or("word"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut recording = words.clone();
    let directory = output.join(format!("record-{index:02}"));
    recording.extend(["--record", directory.to_str().ok_or("record path")?]);
    let exit = if status == 404 || native.get("total") == Some(&json!(0)) {
        1
    } else {
        0
    };
    let delivered = raw_invoke(&env, &recording, exit)?;
    verify(row, &native, &delivered)?;
    restore_date(
        &directory,
        row.get("fetched")
            .and_then(Value::as_u64)
            .ok_or("missing native fetched date")?,
    )?;
    let mut replay = words.clone();
    replay.extend(["--replay", directory.to_str().ok_or("record path")?]);
    let local = raw_invoke(&env, &replay, exit)?;
    verify(row, &native, &local)?;
    mcp(
        &directory,
        &words,
        &local,
        exit,
        Some(native_ceiling(text(row, "body")?)?),
    )?;
    fs::write(
        output.join(format!("{index:02}.json")),
        serde_json::to_vec_pretty(&local).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    use std::io::Write;
    writeln!(
        std::io::stderr(),
        "phenotype native {} full={} comparable={}",
        text(row, "body")?,
        serde_json::to_vec(&local).map_err(|e| e.to_string())?.len(),
        super::super::source_terms::size(&local, "")?
    )
    .map_err(|e| e.to_string())?;
    if stub.finish()?.len() != 1 {
        return Err("native row caused fanout or fallback".into());
    }
    Ok(())
}
}
tool_only! {
fn verify(row: &Value, native: &Value, result: &Value) -> Result<(), String> {
    if row.get("status") == Some(&json!(404)) {
        if result.get("status") != Some(&json!("absent")) {
            return Err("absent source term became failure".into());
        }
        return Ok(());
    }
    if let Some(items) = native.get("items").and_then(Value::as_array) {
        let mapped = result
            .pointer("/result/items")
            .and_then(Value::as_array)
            .ok_or("mapped items")?;
        if items.len() != mapped.len() {
            return Err("native headers dropped".into());
        }
        for (raw, mapped) in items.iter().zip(mapped) {
            for (to, from) in [
                ("id", "id"),
                ("label", "name"),
                ("match_score", "score"),
                ("matched_field", "matched_field"),
                ("match_type", "match_type"),
            ] {
                if mapped.get(to) != raw.get(from).filter(|v| !v.is_null()) {
                    return Err(format!("native search {from} changed"));
                }
            }
        }
        for field in ["limit", "offset", "total"] {
            let path = if field == "limit" {
                "/query/limit".into()
            } else {
                format!("/page/{field}")
            };
            if result.pointer(&path) != native.get(field) {
                return Err(format!("native {field} changed"));
            }
        }
    } else {
        let card = result.pointer("/result/term").ok_or("card")?;
        for field in ["requested_id", "returned_id"] {
            if card.get(field) != native.get("id") {
                return Err("native card identity changed".into());
            }
        }
        let hpo = row.get("source") == Some(&json!("hpo"));
        sections(native, result, hpo)?;
        let mappings = if hpo {
            vec![
                ("label", "name"),
                ("definition", "definition"),
                ("comment", "comment"),
                ("publication_references", "publicationReferences"),
                ("hpo_descendant_count", "descendantCount"),
            ]
        } else {
            vec![
                ("label", "name"),
                ("description", "description"),
                ("deprecated", "deprecated"),
                ("monarch_descendant_count", "has_descendant_count"),
                ("association_counts", "association_counts"),
            ]
        };
        for (to, from) in mappings {
            if to == "association_counts" {
                counts(native, card)?;
                continue;
            }
            if card.get(to) != native.get(from).filter(|v| !v.is_null()) {
                return Err(format!("complete native {from} changed"));
            }
        }
    }
    Ok(())
}
}
tool_only! {
fn counts(native: &Value, card: &Value) -> Result<(), String> {
    let Some(raw) = native
        .get("association_counts")
        .filter(|v| !v.is_null())
        .and_then(Value::as_array)
    else {
        return Ok(());
    };
    let mapped = card
        .get("association_counts")
        .and_then(Value::as_array)
        .ok_or("counts")?;
    if raw.len() != mapped.len() {
        return Err("count scopes changed".into());
    }
    for (raw, mapped) in raw.iter().zip(mapped) {
        for field in [
            "key",
            "category",
            "label",
            "count",
            "count_direct",
            "count_with_orthologs",
        ] {
            if mapped.get(field) != raw.get(field).filter(|v| !v.is_null()) {
                return Err(format!("native count {field} changed"));
            }
        }
    }
    Ok(())
}
}
pub(super) fn mcp(
    directory: &Path,
    words: &[&str],
    local: &Value,
    exit: i32,
    budget: Option<usize>,
) -> Result<(), String> {
    let file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    let ceiling = budget.ok_or("phenotype parity requires a fixed measured ceiling")?;
    fs::write(
        file.path(),
        serde_json::to_vec(&json!(
            {"feature":"phenotype-identity","story":"CLI and MCP preserve the same unchanged native source facts","replay":directory,
            "steps":[{"run":words,"mcp":words,"exit":exit,"is_error":exit>=2,"expect":{"":{"equals":local}},"max_bytes":ceiling,
            "max_tokens":ceiling.div_ceil(4)}]}
        ))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    super::super::run_case(
        file.path(),
        super::super::schema::Registry::embedded()?.as_ref(),
    )?;
    Ok(())
}

pub(super) fn restore_date(directory: &Path, fetched: u64) -> Result<(), String> {
    for source in ["hpo", "monarch"] {
        let path = directory.join(source);
        if !path.exists() {
            continue;
        }
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let mut value: Value =
                serde_json::from_slice(&fs::read(entry.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            for exchange in value
                .get_mut("exchanges")
                .and_then(Value::as_array_mut)
                .ok_or("exchanges")?
            {
                exchange
                    .as_object_mut()
                    .ok_or("exchange")?
                    .insert("fetched".into(), json!(fetched));
            }
            fs::write(
                entry.path(),
                serde_json::to_vec(&value).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

tool_only! {
fn sections(native: &Value, result: &Value, hpo: bool) -> Result<(), String> {
    let present = |field: &str| native.get(field).is_some_and(|v| !v.is_null());
    let mut expected = Vec::new();
    if if hpo {
        present("synonyms")
    } else {
        [
            "synonym",
            "exact_synonym",
            "broad_synonym",
            "narrow_synonym",
            "related_synonym",
        ]
        .into_iter()
        .any(present)
    } {
        expected.push("synonyms");
    }
    if present(if hpo { "xrefs" } else { "xref" }) {
        expected.push("xrefs");
    }
    if !hpo {
        if ["super_classes", "sub_classes"].into_iter().any(|field| {
            native
                .pointer(&format!("/node_hierarchy/{field}"))
                .is_some_and(|v| !v.is_null())
        }) {
            expected.push("hierarchy");
        }
        for (key, section) in [
            ("biolink:DiseaseToPhenotypicFeatureAssociation", "diseases"),
            ("biolink:GeneToPhenotypicFeatureAssociation", "genes"),
        ] {
            if native
                .get("association_counts")
                .and_then(Value::as_array)
                .is_some_and(|rows| rows.iter().any(|row| row.get("key") == Some(&json!(key))))
            {
                expected.push(section);
            }
        }
    }
    if result.pointer("/result/sections") != Some(&json!(expected)) {
        return Err("native available sections changed".into());
    }
    let handlers = expected
        .into_iter()
        .filter(|section| matches!(*section, "synonyms" | "xrefs" | "hierarchy"))
        .collect::<Vec<_>>();
    let next = result
        .get("next")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    if next.len() != handlers.len()
        || next.iter().zip(handlers).any(|(step, section)| {
            step.get("kind") != Some(&json!("section"))
                || step.pointer("/args/3") != Some(&json!(section))
        })
    {
        return Err("card section steps do not match the implemented source fields".into());
    }
    Ok(())
}
}

// Fixed j14607/j14614 comparable CLI/MCP measurements plus fifteen percent.
tool_only! {
fn native_ceiling(body: &str) -> Result<usize, String> {
    let card = match body {
        "00-hpo-term.raw" => Some(("HP_0001250", "hpo")),
        "01-monarch-term.raw" => Some(("HP_0001250", "monarch")),
        "40-obsolete.raw" => Some(("HP_0001676", "monarch")),
        _ => body
            .split_once("-card-")
            .and_then(|(_, id)| id.strip_suffix(".raw"))
            .map(|id| (id, "hpo")),
    };
    if let Some((id, source)) = card {
        return super::section_native::ceiling(&format!("card-{id}-{source}"));
    }
    crate::runner::launch::native_budget(&format!("phenotype/{body}"))
}
}
