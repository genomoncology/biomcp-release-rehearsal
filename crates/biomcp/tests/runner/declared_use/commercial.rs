//! Feature source-controls: commercial exclusions preserve free sources and filter licensed fields.
use crate::runner::launch::Environment;
use serde_json::{Value, json};

pub(super) fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"disable_sources":["noncommercial", "licensed"]}))
        .map_err(|e| e.to_string())?;
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &["health".into(), "--offline".into()],
        )
        .output()
        .map_err(|e| e.to_string())?;
    let reply: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let sources = reply
        .pointer("/result/sources")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("commercial health missing: {reply}"))?;
    for name in [
        "pubmed",
        "ctgov",
        "clinvar",
        "cpic",
        "mutalyzer",
        "variantvalidator",
    ] {
        let source = sources
            .iter()
            .find(|row| row["source"] == name)
            .ok_or_else(|| format!("source absent: {name}"))?;
        if source["disabled"] != false {
            return Err(format!("free source disabled: {name}"));
        }
    }
    for (name, component) in [("myvariant", "cosmic"), ("mydisease", "omim")] {
        let row = sources
            .iter()
            .find(|row| row["source"] == name)
            .and_then(|row| row["components"].as_array())
            .and_then(|rows| rows.iter().find(|row| row["name"] == component))
            .ok_or_else(|| format!("component absent: {name}/{component}"))?;
        if row["disabled"] != true {
            return Err(format!("licensed component enabled: {name}/{component}"));
        }
    }
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &[
                "health".into(),
                "--offline".into(),
                "--disable-sources".into(),
                "restricted".into(),
            ],
        )
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        return Err("retired restricted group accepted".into());
    }
    Ok(())
}
