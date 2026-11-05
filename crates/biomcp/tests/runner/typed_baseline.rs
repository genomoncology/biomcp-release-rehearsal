//! Compare raw single-tool wire replies with the verified unchanged product baseline.
use super::super::super::{
    launch::{self, Environment},
    mcp::request,
    process,
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, process::Stdio};

pub(super) fn check() -> Result<(), String> {
    let baseline = launch::typed_baseline()
        .ok_or("BIOMCP_TYPED_BASELINE is required for single-mode wire parity")?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let frames = [
        request(json!(1), "server/discover", json!({})),
        request(json!(2), "tools/list", json!({})),
        request(
            json!(3),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":["get","variant","BRAF V600E"]}}),
        ),
        request(
            json!(4),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":["search","trial","--condition","melanoma"]}}),
        ),
    ];
    let expected = exchange(
        &env,
        &baseline,
        &["serve".into(), "--offline".into()],
        &frames,
    )?;
    for mode in [
        vec!["serve".into(), "--offline".into()],
        vec![
            "serve".into(),
            "--offline".into(),
            "--tool-mode".into(),
            "single".into(),
        ],
    ] {
        let actual = exchange(
            &env,
            std::path::Path::new(env!("CARGO_BIN_EXE_biomcp")),
            &mode,
            &frames,
        )?;
        compare_frames(&frames, &expected, &actual)?;
    }
    invalid_id_witness()
}
pub(super) fn exchange(
    env: &Environment,
    binary: &std::path::Path,
    args: &[String],
    frames: &[Value],
) -> Result<BTreeMap<String, String>, String> {
    let mut transcript_args = args.to_vec();
    transcript_args.extend(frames.iter().map(Value::to_string));
    let mut command = env.command_with_arguments(binary, args, &transcript_args);
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let (status, stdout, stderr) = process::exchange(child, frames)?;
    command
        .record(status, &stdout, "mcp")
        .map_err(|e| e.to_string())?;
    if !status.success() || !stderr.is_empty() {
        return Err(format!(
            "baseline process failed: {}",
            String::from_utf8_lossy(&stderr)
        ));
    }
    String::from_utf8(stdout)
        .map_err(|e| e.to_string())?
        .lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
            Ok((
                value.get("id").ok_or("missing reply id")?.to_string(),
                line.to_owned(),
            ))
        })
        .collect()
}

fn compare_frames(
    frames: &[Value],
    old: &BTreeMap<String, String>,
    new: &BTreeMap<String, String>,
) -> Result<(), String> {
    if old.keys().ne(new.keys()) {
        return Err("single mode changed reply IDs".into());
    }
    for frame in frames {
        let id = frame.get("id").ok_or("frame id")?.to_string();
        let before = old.get(&id).ok_or("frozen reply")?;
        let after = new.get(&id).ok_or("current reply")?;
        if before == after {
            continue;
        }
        return Err(format!(
            "unapproved single RAW frame {id}: frozen={before}; current={after}"
        ));
    }
    Ok(())
}

pub(super) fn invalid_id_witness() -> Result<(), String> {
    use std::io::Write;
    let baseline = launch::typed_baseline()
        .ok_or("BIOMCP_TYPED_BASELINE is required for single-mode wire parity")?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let frames = [request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":["get","trial","lung cancer"]}}),
    )];
    let args = ["serve".into(), "--offline".into()];
    let old = exchange(&env, &baseline, &args, &frames)?;
    let current = exchange(
        &env,
        std::path::Path::new(env!("CARGO_BIN_EXE_biomcp")),
        &args,
        &frames,
    )?;
    for (label, replies) in [("frozen-ed9c", &old), ("current-default-single", &current)] {
        for raw in replies.values() {
            writeln!(
                std::io::stdout(),
                "TYPED_INVALID_ID_RAW {label} mode=serve-offline isolated-config=default {raw}"
            )
            .map_err(|e| e.to_string())?;
        }
    }
    compare_frames(&frames, &old, &current)
}
