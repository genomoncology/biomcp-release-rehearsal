use super::{Check, Environment, call};
use crate::runner::{launch::fixtures, mcp};
use serde_json::{Value, json};
use std::{collections::HashSet, fs, path::PathBuf};
fn seed(env: &Environment) -> Result<PathBuf, String> {
    let replay = env.root().join("search-replay");
    fs::create_dir_all(replay.join("who-ivd")).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(fixtures().join("replies/diagnostic/who-search-synthetic/who-ivd"))
        .map_err(|e| e.to_string())?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        fs::copy(entry.path(), replay.join("who-ivd").join(entry.file_name()))
            .map_err(|e| e.to_string())?;
    }
    Ok(replay)
}
fn args(step: &Value) -> Result<Vec<String>, String> {
    serde_json::from_value(step.get("args").cloned().ok_or("missing args")?)
        .map_err(|e| e.to_string())
}

pub(super) fn walk() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env)?;
    call(&env, &replay, &["sync", "who-ivd"], 0)?;
    fs::remove_dir_all(replay.join("who-ivd")).map_err(|e| e.to_string())?;
    let mut command: Vec<String> = [
        "search",
        "diagnostic",
        "--source",
        "who-ivd",
        "--marker",
        "HIV",
    ]
    .map(str::to_owned)
    .into();
    let mut seen = Vec::new();
    loop {
        let borrowed: Vec<_> = command.iter().map(String::as_str).collect();
        let value = call(&env, &replay, &borrowed, 0)?;
        let globals = env.globals(Some(&replay), false);
        if mcp::call_with_globals(&env, &command, false, &globals)? != value {
            return Err("WHO page differs across CLI and MCP".into());
        }
        let items = value
            .pointer("/result/items")
            .and_then(Value::as_array)
            .ok_or("items missing")?;
        let next = value
            .get("next")
            .and_then(Value::as_array)
            .ok_or("actions missing")?;
        let candidates: Vec<_> = next
            .iter()
            .filter(|v| v.get("kind") == Some(&json!("disambiguate")))
            .collect();
        if candidates.len() != items.len() || value.pointer("/page/total") != Some(&json!(7)) {
            return Err(format!("lost candidate actions or total: {value}"));
        }
        for (item, step) in items.iter().zip(candidates) {
            let id = item.get("id").and_then(Value::as_str).ok_or("ID missing")?;
            seen.push(id.to_owned());
            let get = args(step)?;
            if !get.windows(2).any(|v| v == ["--source", "who-ivd"])
                || !get.iter().any(|v| v == "--bundle-revision")
            {
                return Err(format!("get lost source/revision: {step}"));
            }
            let borrowed: Vec<_> = get.iter().map(String::as_str).collect();
            let card = call(&env, &replay, &borrowed, 0)?;
            if card.pointer("/result/id") != Some(&json!(id)) {
                return Err("get action changed identity".into());
            }
        }
        if let Some(step) = next.iter().find(|v| v.get("kind") == Some(&json!("page"))) {
            if !args(step)?.windows(2).any(|v| v == ["--marker", "HIV"]) {
                return Err("continuation lost selector".into());
            }
            command = args(step)?;
        } else {
            break;
        }
    }
    let expected = [
        "0466-046-00",
        "1001-001-00",
        "1002-001-00",
        "1003-001-00",
        "1004-001-00",
        "1005-001-00",
        "1006-001-00",
    ];
    if seen != expected || seen.iter().collect::<HashSet<_>>().len() != 7 {
        return Err(format!(
            "WHO product walk has gaps, duplicates or reordering: {seen:?}"
        ));
    }
    Ok(())
}
