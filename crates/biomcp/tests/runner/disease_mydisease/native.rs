//! Explicit command proof reads unchanged retained bodies outside Git.
use super::{Stub, setup};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};

pub(in crate::runner) fn run(root: &Path) -> Result<(), String> {
    let requests = load(root)?;
    let mut replies = BTreeMap::new();
    for row in &requests {
        let id = row["requested_id"].as_str().ok_or("missing requested ID")?;
        let bytes = fs::read(
            root.join(row["directory"].as_str().ok_or("missing directory")?)
                .join("body.raw"),
        )
        .map_err(|e| e.to_string())?;
        if digest(&bytes) != row["sha256"].as_str().ok_or("missing hash")? {
            return Err("native body hash changed".into());
        }
        replies.insert(
            id.to_owned(),
            String::from_utf8(bytes).map_err(|e| e.to_string())?,
        );
    }
    let stub = Stub::for_source_status(
        "mydisease.info",
        "application/json",
        move |request| {
            let target = request
                .lines()
                .next()
                .and_then(|v| v.split_whitespace().nth(1))
                .ok_or("missing target")?;
            let (path, query) = target.split_once('?').ok_or("missing projection")?;
            if query != "fields=mondo%2Cdisease_ontology" {
                return Err("changed projection".into());
            }
            let id = path
                .strip_prefix("/v1/disease/")
                .ok_or("wrong endpoint")?
                .replace("%3A", ":");
            replies.get(&id).cloned().ok_or("undeclared request".into())
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    for (index, row) in requests.iter().enumerate() {
        prove_row(root, &env, index, row)?;
    }
    if stub.finish()?.len() != 11 {
        return Err("unexpected native command request count".into());
    }
    Ok(())
}
fn prove_row(
    root: &Path,
    env: &super::Environment,
    index: usize,
    row: &Value,
) -> Result<(), String> {
    let id = row["requested_id"].as_str().ok_or("missing ID")?;
    let words = ["get", "disease", id];
    let output = root.join("product-proof");
    fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let directory = output.join(format!("native-{index}"));
    let mut command = vec!["--record".into(), directory.to_string_lossy().into_owned()];
    command.extend(super::args(&words));
    let value = super::super::proof::retained_cli(
        env,
        &command,
        0,
        &output.join(format!("delivery-{index}")),
    )?;
    let body: Value = serde_json::from_slice(
        &fs::read(
            root.join(row["directory"].as_str().ok_or("missing directory")?)
                .join("body.raw"),
        )
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let mut expected = json!({"/status":"complete",
"/result/requested_id":id,
"/result/returned_id":row["returned_id"],
"/result/concept/label":body.pointer("/mondo/label").ok_or("missing source label")?});
    if let Some(definition) = body.pointer("/mondo/definition") {
        super::insert(&mut expected, "/result/definition", definition.clone())?;
    } else if value.pointer("/result/definition").is_some() {
        return Err("absent native definition was filled".into());
    }
    if id == "DOID:3910" {
        super::insert(&mut expected, "/result/alias/object/id", json!(id))?;
    }
    if id == "MONDO:0007607" {
        super::insert(
            &mut expected,
            "/result/consider",
            json!(["MONDO:0800444", "MONDO:0800445"]),
        )?;
        super::insert(&mut expected, "/result/sections", json!([]))?;
    }
    replay(
        env,
        &directory,
        json!([{"run":words,"mcp":words,"expect":expected}]),
    )?;
    if matches!(
        id,
        "DOID:3910" | "MONDO:0005105" | "MONDO:0018875" | "MONDO:0800444" | "MONDO:0007607"
    ) {
        sections(env, &directory, id, &body)?;
    }
    Ok(())
}
fn sections(
    env: &super::Environment,
    directory: &Path,
    id: &str,
    body: &Value,
) -> Result<(), String> {
    for section in ["synonyms", "xrefs", "hierarchy"] {
        let total = count(body, section);
        let offsets = if total > 2 {
            vec![0, total - 2]
        } else {
            vec![0]
        };
        for offset in offsets {
            let offset_string = offset.to_string();
            let words = [
                "get",
                "disease",
                id,
                section,
                "--section-limit",
                "2",
                "--section-offset",
                &offset_string,
            ];
            let exit = if total == 0 { 1 } else { 0 };
            let value = replay(
                env,
                directory,
                json!([{"run":words,"mcp":words,"exit":exit,"expect":{"/page/offset":offset,"/page/total":total,"/page/returned":2.min(total)}}]),
            )?;
            super::measure(&words, &value)?;
            if offset + 2 >= total && value.get("next").is_some() {
                return Err("terminal native section advertised continuation".into());
            }
            if id == "MONDO:0018875"
                && section == "xrefs"
                && offset == 0
                && (value
                    .pointer("/result/items/0/object/id")
                    .and_then(Value::as_str)
                    != Some("DOID:3012")
                    || value
                        .pointer("/result/items/1/object/id")
                        .and_then(Value::as_str)
                        != Some("DOID:0111503"))
            {
                return Err("multiple DOID choices were lost".into());
            }
        }
    }
    if id == "DOID:3910" {
        default_pages(env, directory, id, body)?;
    }
    let mut command = super::args(&["get", "disease", id]);
    command.extend([
        "--replay".into(),
        directory.to_string_lossy().into(),
        "--format".into(),
        "md".into(),
    ]);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &command)
        .output()
        .map_err(|e| e.to_string())?;
    let retained = directory
        .parent()
        .ok_or("missing proof directory")?
        .join(format!("{id}-markdown"));
    fs::create_dir(&retained).map_err(|e| e.to_string())?;
    fs::write(retained.join("cli.stdout"), &output.stdout).map_err(|e| e.to_string())?;
    fs::write(retained.join("cli.stderr"), &output.stderr).map_err(|e| e.to_string())?;
    if !output.status.success()
        || !String::from_utf8_lossy(&output.stdout)
            .contains(&super::super::source_terms::text::escape(id))
    {
        return Err("Markdown lost exact requested identity".into());
    }
    Ok(())
}
fn count(body: &Value, section: &str) -> usize {
    ["mondo", "disease_ontology"]
        .iter()
        .map(|component| {
            let object = &body[component];
            match section {
                "hierarchy" => object["parents"].as_array().map_or(0, Vec::len),
                "synonyms" => object[if *component == "mondo" {
                    "synonym"
                } else {
                    "synonyms"
                }]
                .as_object()
                .map_or(0, |v| v.values().map(values).sum()),
                _ => object["xrefs"]
                    .as_object()
                    .map_or(0, |v| v.values().map(values).sum()),
            }
        })
        .sum()
}
fn values(value: &Value) -> usize {
    value
        .as_array()
        .map_or(usize::from(value.is_string()), Vec::len)
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect()
}
fn load(root: &Path) -> Result<Vec<Value>, String> {
    let bytes = fs::read(root.join("requests.json")).map_err(|e| e.to_string())?;
    if digest(&bytes) != "7cd1db0335a1cb96203260f4ea7887b54ebdc9e94df9f3ee8c3014f063f106ba" {
        return Err("native manifest hash changed".into());
    }
    let requests: Vec<Value> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    // URLs, original retrieval dates and hashes stay in this manifest. Runtime delivery is local.
    Ok(requests
        .into_iter()
        .filter(|v| v["source"] == "mydisease")
        .collect())
}

fn default_pages(
    env: &super::Environment,
    directory: &Path,
    id: &str,
    body: &Value,
) -> Result<(), String> {
    for section in ["synonyms", "xrefs", "hierarchy"] {
        let total = count(body, section);
        for limit in [5usize, 50] {
            let limit_string = limit.to_string();
            let words = [
                "get",
                "disease",
                id,
                section,
                "--section-limit",
                &limit_string,
            ];
            let value = replay(
                env,
                directory,
                json!([{"run":words,"mcp":words,"expect":{"/page/returned":limit.min(total),"/page/total":total}}]),
            )?;
            super::measure(&words, &value)?;
        }
    }
    Ok(())
}



fn replay(env: &super::Environment, recording: &Path, steps: Value) -> Result<Value, String> {
    let output = recording
        .parent()
        .ok_or("missing external proof directory")?;
    let mut last = None;
    for mut value in steps.as_array().ok_or("missing steps")?.clone() {
        let ceiling = super::ceiling(&value)?;
        super::insert(&mut value, "max_bytes", json!(ceiling))?;
        super::insert(&mut value, "max_tokens", json!(ceiling.div_ceil(4)))?;
        let step: super::super::Step = serde_json::from_value(value).map_err(|e| e.to_string())?;
        let mut args = vec!["--replay".into(), recording.to_string_lossy().into_owned()];
        args.extend(step.run.clone().ok_or("missing command")?);
        let index = fs::read_dir(output)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .filter(|v| v.file_name().to_string_lossy().starts_with("replay-"))
            .count();
        let directory = output.join(format!("replay-{index}"));
        let actual = super::super::proof::retained_both(env, &args, step.exit, &directory)?;
        super::super::assertions::check(
            &fs::read(directory.join("cli.stdout")).map_err(|e| e.to_string())?,
            &step,
            &super::super::assertions::prepare(&step)?,
        )?;
        last = Some(actual);
    }
    last.ok_or("missing native response".into())
}
