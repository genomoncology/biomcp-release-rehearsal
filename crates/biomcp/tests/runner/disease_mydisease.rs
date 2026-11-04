//! Feature disease-mydisease: exact annotation without fallback or hidden enrichment.
use super::{launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::fs;

tool_only! { mod native; }
tool_only! { pub(super) use native::run as native; }

fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("mydisease-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|s| (*s).into()).collect()
}
fn insert(value: &mut Value, key: &str, item: Value) -> Result<(), String> {
    value
        .as_object_mut()
        .ok_or("missing object")?
        .insert(key.into(), item);
    Ok(())
}

fn ceiling(step: &Value) -> Result<usize, String> {
    // Ceil(max measured comparable bytes by command/page shape * 1.15), j12127/j12134.
    let words = step
        .get("run")
        .and_then(Value::as_array)
        .ok_or("missing command")?;
    let section = words.get(3).and_then(Value::as_str);
    let limit = words.windows(2).find_map(|w| {
        let (key, value) = (w.first()?.as_str()?, w.get(1)?.as_str()?);
        (key == "--section-limit").then_some(value)
    });
    Ok(match (section, limit) {
        (Some("synonyms"), Some("2")) => {
            crate::runner::output_limit("disease_mydisease-measured-1237")?
        }
        (Some("xrefs"), Some("2")) => {
            crate::runner::output_limit("disease_mydisease-measured-1374")?
        }
        (Some("hierarchy"), Some("2")) => {
            crate::runner::output_limit("disease_mydisease-measured-1385")?
        }
        (Some("synonyms"), Some("50")) => {
            crate::runner::output_limit("disease_mydisease-measured-2230")?
        }
        (Some("xrefs"), Some("50")) => {
            crate::runner::output_limit("disease_mydisease-measured-5619")?
        }
        (Some("hierarchy"), Some("50")) => {
            crate::runner::output_limit("disease_mydisease-measured-1796")?
        }
        (Some("synonyms"), _) => crate::runner::output_limit("disease_mydisease-measured-1889")?,
        (Some("xrefs"), _) => crate::runner::output_limit("disease_mydisease-measured-2200")?,
        (Some("hierarchy"), _) => crate::runner::output_limit("disease_mydisease-measured-1794")?,
        _ => crate::runner::output_limit("disease_mydisease-measured-2024")?,
    })
}

fn measure(words: &[&str], value: &Value) -> Result<(), String> {
    use std::io::Write;
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    writeln!(
        std::io::stderr(),
        "MyDisease {:?}: full={} comparable={}",
        words,
        bytes,
        super::source_terms::size(value, "")?
    )
    .map_err(|e| e.to_string())
}

tool_only! {}
