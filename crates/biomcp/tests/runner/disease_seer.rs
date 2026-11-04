//! Feature disease-seer: native site choices and independent annual statistics.
use super::{launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path};

tool_only! { mod native; }
tool_only! { pub(super) use native::run as native; }

fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("seer-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| (*s).into()).collect()
}
fn invoke(env: &Environment, words: &[String], exit: i32) -> Result<Value, String> {
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), words)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) {
        return Err(format!(
            "wrong exit {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    let value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    measure(
        &words.iter().map(String::as_str).collect::<Vec<_>>(),
        &value,
    )?;
    Ok(value)
}
fn record(env: &Environment, words: &[&str], directory: &Path, exit: i32) -> Result<Value, String> {
    let mut command = args(words);
    command.extend(["--record".into(), directory.to_string_lossy().into()]);
    let result = invoke(env, &command, exit)?;
    Ok(result)
}
fn measure(words: &[&str], result: &Value) -> Result<(), String> {
    let full = serde_json::to_vec(result).map_err(|e| e.to_string())?.len();
    let comparable = super::source_terms::size(result, "")?;
    writeln!(
        std::io::stderr(),
        "SEER {:?}: full={full}, comparable={comparable}, estimated tokens={}",
        words,
        comparable.div_ceil(4)
    )
    .map_err(|e| e.to_string())
}
fn replay(directory: &Path, steps: Value) -> Result<Value, String> {
    let file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    fs::write(
        file.path(),
        serde_json::to_vec(&json!(
            {"feature":"disease-seer","story":"Native site and statistic selectors retain source values","replay":directory,"steps":steps}
        ))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let value = super::run_case(file.path(), super::schema::Registry::embedded()?.as_ref())?;
    measure(&["replay-terminal"], &value)?;
    Ok(value)
}

fn markdown(
    env: &Environment,
    words: &[&str],
    directory: &Path,
    required: &[&str],
) -> Result<(), String> {
    let mut command = args(words);
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
    let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    if !output.status.success() || required.iter().any(|r| !text.contains(r)) {
        return Err(format!("Markdown lost SEER facts: {text}"));
    }
    Ok(())
}
