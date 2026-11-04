//! Feature cell-line-identity: unchanged retained native requests through the real CLI and MCP.
use std::io::Write;
mod chembl;
tool_only! { mod examples; }
mod faults;
mod help;
mod native;
mod research;
mod sections;
use super::{launch::Environment, network_stub::Stub};
use serde_json::{Value, json};
use std::fs;

fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("cellosaurus-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn invoke(env: &Environment, words: &[&str], exit: i32) -> Result<Value, String> {
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &words.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) {
        return Err(format!(
            "{words:?}: exit {:?}; stdout {}; stderr {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
fn parity(
    phase: &str,
    env: &Environment,
    words: &[&str],
    exit: i32,
    measure: bool,
) -> Result<Value, String> {
    let recording = env.root().join("recordings").join(phase.replace(':', "_"));
    fs::create_dir_all(recording.parent().ok_or("record parent")?).map_err(|e| e.to_string())?;
    let mut args = words.to_vec();
    args.extend(["--record", recording.to_str().ok_or("recording path")?]);
    let value = invoke(env, &args, exit)?;
    compared(phase, env, words, exit, measure, &recording, value)
}
fn compared(
    phase: &str,
    env: &Environment,
    words: &[&str],
    exit: i32,
    measure: bool,
    recording: &std::path::Path,
    value: Value,
) -> Result<Value, String> {
    let replay: Vec<String> = [
        "--replay".to_string(),
        recording.to_str().ok_or("recording path")?.into(),
    ]
    .into();
    let mcp = super::mcp::call_with_globals(
        env,
        &words.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        exit >= 2,
        &replay,
    )?;
    if mcp != value {
        return Err(format!("{phase}: CLI/MCP mismatch"));
    }
    for (surface, actual) in [("cli", &value), ("mcp", &mcp)] {
        let bytes = serde_json::to_vec(actual).map_err(|e| e.to_string())?.len()
            + usize::from(surface == "cli");
        let _ = writeln!(
            std::io::stderr(),
            "CELL_LINE_MEASURE {phase} [{surface}] {bytes}"
        );
        if !measure {
            budget(phase, surface, actual)?;
        }
    }
    if matches!(phase, "native/search/MOLM13" | "native/card/CVCL_0007") {
        markdown(phase, env, words, &replay, &value, measure)?;
    }
    if phase.starts_with("sections/") {
        sections::markdown(phase, env, words, &replay, &value, measure)?;
    }
    let registry = super::schema::Registry::embedded()?;
    let arguments = words.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    registry.validate(&value, &arguments, exit == 0)?;
    registry.validate(&mcp, &arguments, exit == 0)?;
    Ok(value)
}
fn ceiling(phase: &str, surface: &str) -> Result<usize, String> {
    let measured: std::collections::BTreeMap<String, usize> = serde_saphyr::from_str(
        &crate::runner::launch::fixture_text("cell-line-command-budgets.yaml")?,
    )
    .map_err(|e| e.to_string())?;
    measured
        .get(&format!("{phase} [{surface}]"))
        .map(|v| (v * 115).div_ceil(100))
        .ok_or_else(|| format!("unknown cell line phase: {phase} [{surface}]"))
}
fn budget(phase: &str, surface: &str, value: &Value) -> Result<(), String> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| e.to_string())?.len() + usize::from(surface == "cli");
    let _ = writeln!(
        std::io::stderr(),
        "CELL_LINE_MEASURE {phase} [{surface}] {bytes}"
    );
    if bytes > ceiling(phase, surface)? {
        return Err(format!(
            "{phase} [{surface}] exceeds measured output budget"
        ));
    }
    Ok(())
}
fn fixture(name: &str) -> Result<String, String> {
    fs::read_to_string(
        super::launch::fixtures()
            .join("fixtures/cell-line/native")
            .join(name),
    )
    .map_err(|e| e.to_string())
}
fn canonical(target: &str) -> (String, Vec<&str>) {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut params: Vec<_> = query.split('&').filter(|p| !p.is_empty()).collect();
    params.sort_unstable();
    (path.into(), params)
}
fn recorded_stub() -> Result<Stub, String> {
    let requests: std::collections::BTreeMap<String, Value> =
        serde_json::from_str(&fixture("requests.json")?).map_err(|e| e.to_string())?;
    let mut replies = Vec::new();
    let mut statuses = std::collections::BTreeMap::new();
    for row in requests.values() {
        let target = row
            .get("target")
            .and_then(Value::as_str)
            .ok_or("target")?
            .to_string();
        replies.push((
            target.clone(),
            fixture(row.get("file").and_then(Value::as_str).ok_or("file")?)?,
        ));
        statuses.insert(
            target,
            row.get("status").and_then(Value::as_u64).ok_or("status")? as u16,
        );
    }
    Stub::for_source_status(
        "api.cellosaurus.org",
        "application/json",
        move |request| {
            let target = request.split_whitespace().nth(1).ok_or("target")?;
            replies
                .iter()
                .find(|(native, _)| canonical(native) == canonical(target))
                .map(|(_, body)| body.clone())
                .ok_or_else(|| format!("Unproved native request: {target}"))
        },
        move |request| {
            let target = request.split_whitespace().nth(1).unwrap_or_default();
            statuses
                .iter()
                .find(|(native, _)| canonical(native) == canonical(target))
                .map(|(_, status)| *status)
                .unwrap_or(500)
        },
    )
}
fn path(value: &Value, path: &str, mut expected: Value) -> Result<(), String> {
    if expected == serde_json::json!([]) && value.pointer(path).is_none() {
        return Ok(());
    }
    super::assertions::omit_empty_lists(&mut expected);
    if value.pointer(path) != Some(&expected) {
        return Err(format!(
            "{path}: expected {expected}, got {:?}",
            value.pointer(path)
        ));
    }
    Ok(())
}

tool_only! {
pub(crate) fn measure() -> Result<(), String> {
    native::run(true)?;
    faults::run(true)?;
    help::run(true)?;
    research::run(true)
}
}

fn markdown(
    phase: &str,
    env: &Environment,
    words: &[&str],
    replay: &[String],
    value: &Value,
    measure: bool,
) -> Result<(), String> {
    let mut args = words.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    args.extend_from_slice(replay);
    args.extend(["--format".into(), "md".into()]);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("native Markdown failed".into());
    }
    let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let _ = writeln!(
        std::io::stderr(),
        "CELL_LINE_MEASURE {phase} [markdown] {}",
        text.len()
    );
    if !measure && text.len() > ceiling(phase, "markdown")? {
        return Err("native Markdown exceeds measured output cap".into());
    }
    let result = value.get("result").ok_or("result")?;
    let header = result
        .get("header")
        .or_else(|| result.pointer("/items/0"))
        .ok_or("header")?;
    for field in ["accession", "name"] {
        if !text.contains(&super::source_terms::text::escape(
            header
                .get(field)
                .and_then(Value::as_str)
                .ok_or("header fact")?,
        )) {
            return Err(format!("Markdown lost {field}"));
        }
    }
    if let Some(warning) = result
        .pointer("/warnings/items/0/text")
        .and_then(Value::as_str)
        && !text.contains(&super::source_terms::text::escape(warning))
    {
        return Err("Markdown shortened qualified warning".into());
    }
    let actions = value
        .get("next")
        .and_then(Value::as_array)
        .ok_or("actions")?;
    for action in actions {
        let args = action
            .get("args")
            .and_then(Value::as_array)
            .ok_or("args")?
            .iter()
            .map(|v| v.as_str().ok_or("arg"))
            .collect::<Result<Vec<_>, _>>()?;
        if !text.contains(&args.join(" ")) {
            return Err("Markdown lost next command".into());
        }
    }
    Ok(())
}

tool_only! {
pub(crate) fn generate_examples(directory: &std::path::Path) -> Result<(), String> {
    examples::generate(directory)
}
}

tool_only! {
pub(crate) fn measure_remaining() -> Result<(), String> {
    native::remaining()?;
    faults::run(true)?;
    help::run(true)?;
    research::run(true)
}
}

tool_only! {
pub(crate) fn measure_canonical() -> Result<(), String> {
    native::canonical(true)
}
}

tool_only! {
pub(crate) fn measure_sections() -> Result<(), String> {
    sections::run(true)
}
}

tool_only! {
pub(crate) fn generate_xrefs(directory: &std::path::Path) -> Result<(), String> {
    examples::generate_xrefs(directory)
}
}

tool_only! {
pub(crate) fn measure_section_release() -> Result<(), String> {
    sections::measure_release()
}
}

tool_only! {
pub(crate) fn measure_help() -> Result<(), String> {
    help::run(true)
}
}
