//! Walk the installed parser's scoped help without maintaining a command inventory.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn collect(
    mut invoke: impl FnMut(Vec<String>) -> Result<Value, String>,
) -> Result<Vec<Value>, String> {
    let root = invoke(vec!["help".into()])?;
    let mut pending = paths(&root)?;
    let mut seen = BTreeSet::new();
    let mut commands = BTreeMap::new();
    while let Some(path) = pending.pop() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let response = invoke(std::iter::once("help".into()).chain(path.clone()).collect())?;
        if path.len() == 1
            && serde_json::to_vec(&response)
                .map_err(|e| e.to_string())?
                .len()
                >= 8192
        {
            return Err(format!("command help exceeds 8 KB: {path:?}"));
        }
        for entry in response
            .pointer("/result/commands")
            .and_then(Value::as_array)
            .ok_or("scoped help commands missing")?
        {
            let Some(command_path) = entry.get("command") else {
                continue;
            };
            let words: Vec<String> =
                serde_json::from_value(command_path.clone()).map_err(|e| e.to_string())?;
            if words == path {
                commands.insert(words, entry.clone());
            } else if words.starts_with(&path) && words.len() == path.len() + 1 {
                pending.push(words);
            }
        }
    }
    let globals = commands
        .get(&vec!["help".to_owned()])
        .and_then(|c| c.get("arguments"))
        .cloned()
        .ok_or("global options missing")?;
    commands.insert(Vec::new(), json!({"command":[],"arguments":globals}));
    Ok(commands.into_values().collect())
}
fn paths(response: &Value) -> Result<Vec<Vec<String>>, String> {
    response
        .pointer("/result/commands")
        .and_then(Value::as_array)
        .ok_or("help commands missing")?
        .iter()
        .filter_map(|c| c.get("command"))
        .map(|path| serde_json::from_value(path.clone()).map_err(|e| e.to_string()))
        .collect()
}
