//! JSON consumers keep reading recorded replies when a major gains fields or enum names.
use super::schema::Registry;
use serde_json::{Value, json};

pub(super) fn recorded_reply(registry: &Registry) -> Result<(), String> {
    let environment = super::launch::Environment::new().map_err(|error| error.to_string())?;
    let args = vec![
        "--replay".into(),
        super::launch::fixtures()
            .join("replies/gene/main")
            .to_string_lossy()
            .into_owned(),
        "search".into(),
        "gene".into(),
        "KRAS".into(),
    ];
    let output = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("recorded gene search failed".into());
    }
    let mut reply: Value =
        serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    registry.validate(&reply, &args, false)?;
    let source = reply
        .pointer_mut("/sources/0")
        .ok_or("missing recorded source")?;
    source["future_property"] = json!({"value":true});
    // Rust's existing reader ignores unknown fields but keeps its own enum vocabulary.
    let _: biomcp_types::SourceOutcome =
        serde_json::from_value(source.clone()).map_err(|error| error.to_string())?;
    source["source"] = json!("future_source");
    reply.pointer_mut("/result").ok_or("missing result")?["future_property"] = json!(true);
    reply
        .pointer_mut("/result/items/0")
        .ok_or("missing recorded gene")?["future_property"] = json!(true);
    registry.validate(&reply, &args, false)
}
