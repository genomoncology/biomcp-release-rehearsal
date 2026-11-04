//! Feature mcp-hosted: the retired CLI group keeps every former exclusion.
use crate::runner::launch::Environment;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

pub(super) fn check() -> Result<(), String> {
    let Some(baseline) = std::env::var_os("BIOMCP_LEGACY_EXECUTABLE") else {
        return Ok(());
    };
    let env = Environment::new().map_err(|e| e.to_string())?;
    let args = ["health", "--offline", "--disable-sources", "restricted"].map(str::to_owned);
    let old = read(&env, Path::new(&baseline), &args)?;
    let current = read(&env, Path::new(env!("CARGO_BIN_EXE_biomcp")), &args)?;
    let before = membership(&old)?;
    let after = membership(&current)?;
    for (name, disabled) in &before {
        if after.get(name) != Some(disabled) {
            return Err(format!("legacy restricted membership changed for {name}"));
        }
    }
    if after
        .iter()
        .any(|(name, disabled)| !before.contains_key(name) && *disabled)
    {
        return Err("legacy group disabled a new component".into());
    }
    Ok(())
}
fn read(env: &Environment, binary: &Path, args: &[String]) -> Result<Value, String> {
    let output = env
        .command(binary, args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.stderr.is_empty() {
        return Err("health wrote diagnostics".into());
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
fn membership(reply: &Value) -> Result<BTreeMap<String, bool>, String> {
    let mut result = BTreeMap::new();
    for source in reply
        .pointer("/result/sources")
        .and_then(Value::as_array)
        .ok_or("health sources missing")?
    {
        let name = source["source"].as_str().ok_or("source name missing")?;
        result.insert(
            name.into(),
            source["disabled"]
                .as_bool()
                .ok_or("source policy missing")?,
        );
        for component in source["components"].as_array().into_iter().flatten() {
            let component_name = component["name"].as_str().ok_or("component name missing")?;
            result.insert(
                format!("{name}/{component_name}"),
                component["disabled"]
                    .as_bool()
                    .ok_or("component policy missing")?,
            );
        }
    }
    Ok(result)
}
