//! Feature phenotype-identity: literal candidates and independent source term cards.

mod budgets;

tool_only! { mod native; }

tool_only! { mod section_native; }
mod sections;

use super::{launch::Environment, network_stub::Stub};
use libtest_mimic::Trial;
tool_only! { pub(super) use native::run as native; }
use serde_json::{Value, json};
use std::fs;
pub(super) fn trials() -> Vec<Trial> {
    sections::trials()
}
fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("phenotype-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn invoke(phase: &str, env: &Environment, words: &[&str], exit: i32) -> Result<Value, String> {
    let value = raw_invoke(env, words, exit)?;
    budgets::check(&value, phase, "cli")?;
    Ok(value)
}
fn raw_invoke(env: &Environment, words: &[&str], exit: i32) -> Result<Value, String> {
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &words.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) {
        return Err(format!(
            "exit {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    let value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    Ok(value)
}
