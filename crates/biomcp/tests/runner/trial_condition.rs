//! Exact trial disease identity stops before trial I/O when resolution cannot supply a filter.
use super::{launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::fs;

tool_only! { mod native; }
tool_only! { pub(super) use native::run as native; }

fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!(
        {"proxy":format!("http://{}",stub.address),"ca_bundle":ca,"keys":{"nci_api_key":"synthetic-trial-condition-key"}}
    ))
    .map_err(|e| e.to_string())?;
    Ok(env)
}
fn invoke(env: &Environment, args: &[String], exit: i32) -> Result<Value, String> {
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), args)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) {
        return Err(format!(
            "expected exit {exit}, got {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
