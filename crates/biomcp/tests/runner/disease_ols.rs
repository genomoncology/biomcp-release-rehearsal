//! Feature disease-ols: literal ontology discovery and exact source concepts.
use super::{launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::io::Write;
use std::{fs, path::Path};
tool_only! { mod native; }
tool_only! { pub(super) use native::run as native; }

fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("ols-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
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
            "wrong exit {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| (*s).into()).collect()
}
fn record(env: &Environment, words: &[&str], directory: &Path) -> Result<Value, String> {
    let mut command = args(words);
    command.extend(["--record".into(), directory.to_string_lossy().into()]);
    let result = invoke(env, &command, 0)?;
    writeln!(
        std::io::stderr(),
        "disease measurement {:?}: {} JSON bytes",
        words,
        serde_json::to_vec(&result)
            .map_err(|e| e.to_string())?
            .len()
    )
    .map_err(|e| e.to_string())?;
    Ok(result)
}
fn replay(directory: &Path, steps: Value) -> Result<Value, String> {
    let file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    fs::write(
        file.path(),
        serde_json::to_vec(&json!(
            {"feature":"disease-ols","story":"Source disease facts retain identity, order and pagination","replay":directory,"steps":steps}
        ))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let value = super::run_case(file.path(), super::schema::Registry::embedded()?.as_ref())?;
    writeln!(
        std::io::stderr(),
        "OLS replay: full={} comparable={}",
        serde_json::to_vec(&value).map_err(|e| e.to_string())?.len(),
        super::source_terms::size(&value, "")?
    )
    .map_err(|e| e.to_string())?;
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
    if !output.status.success() || required.iter().any(|value| !text.contains(value)) {
        return Err(format!("Markdown lost source facts: {text}"));
    }
    Ok(())
}
