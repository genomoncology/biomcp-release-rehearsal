//! Produce executable examples from the same unchanged native bodies as identity cases.
use super::{Value, Write, fixture, fs, invoke, json, path, recorded_stub, setup};
use std::path::Path;
pub(in crate::runner) fn generate(directory: &Path) -> Result<(), String> {
    if directory.exists() {
        return Err("example output already exists".into());
    }
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    for (index, words) in [
        ["search", "cell-line", "MOLM13"],
        ["search", "cell-line", "MV4;11"],
        ["get", "cell-line", "CVCL_2119"],
        ["get", "cell-line", "CVCL_0004"],
    ]
    .into_iter()
    .enumerate()
    {
        let recording = env.root().join(format!("example-{index}"));
        let mut args = words.to_vec();
        args.extend(["--record", recording.to_str().ok_or("record path")?]);
        let value = invoke(&env, &args, 0)?;
        let replay = [
            "--replay".into(),
            recording.to_str().ok_or("record path")?.into(),
        ];
        let mcp =
            super::super::mcp::call_with_globals(&env, &words.map(String::from), false, &replay)?;
        if value != mcp {
            return Err("example CLI/MCP mismatch".into());
        }
        for (surface, value) in [("cli", &value), ("mcp", &mcp)] {
            let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len()
                + usize::from(surface == "cli");
            writeln!(
                std::io::stderr(),
                "CELL_LINE_EXAMPLE {index} [{surface}] {bytes}"
            )
            .map_err(|e| e.to_string())?;
        }
        merge(&recording, directory)?;
    }
    stub.finish()?;
    Ok(())
}
fn merge(recording: &Path, target: &Path) -> Result<(), String> {
    for source in fs::read_dir(recording).map_err(|e| e.to_string())? {
        let source = source.map_err(|e| e.to_string())?;
        let destination = target.join(source.file_name());
        fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
        for file in fs::read_dir(source.path()).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            let destination = destination.join(file.file_name());
            if !destination.exists() {
                fs::copy(file.path(), destination).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

pub(in crate::runner) fn generate_xrefs(directory: &Path) -> Result<(), String> {
    if directory.exists() {
        return Err("xref output already exists".into());
    }
    let stub = recorded_stub()?;
    let env = setup(&stub)?;
    let recording = env.root().join("xref-first");
    let words = [
        "get",
        "cell-line",
        "CVCL_0004",
        "xrefs",
        "--section-limit",
        "25",
    ];
    let mut args = words.to_vec();
    args.extend(["--record", recording.to_str().ok_or("record path")?]);
    let first = invoke(&env, &args, 0)?;
    let replay = [
        "--replay".to_string(),
        recording.to_str().ok_or("record path")?.into(),
    ];
    let mut value = first;
    let mut all = Vec::new();
    let mut maximum = 0;
    for offset in (0..1398).step_by(25) {
        let offset_text = offset.to_string();
        let mut words = words.to_vec();
        if offset > 0 {
            words.extend(["--section-offset", &offset_text]);
            let mut args = words.clone();
            args.extend(["--replay", recording.to_str().ok_or("record path")?]);
            value = invoke(&env, &args, 0)?;
        }
        let mcp = super::super::mcp::call_with_globals(
            &env,
            &words.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            false,
            &replay,
        )?;
        if mcp != value {
            return Err("xref walk CLI/MCP mismatch".into());
        }
        path(&value, "/result/items/total", json!(1398))?;
        path(&value, "/result/offset", json!(offset))?;
        for (surface, value) in [("cli", &value), ("mcp", &mcp)] {
            let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len()
                + usize::from(surface == "cli");
            maximum = maximum.max(bytes);
            writeln!(
                std::io::stderr(),
                "CELL_LINE_XREF_WALK {offset} [{surface}] {bytes}"
            )
            .map_err(|e| e.to_string())?;
        }
        all.extend(
            value
                .pointer("/result/items/items")
                .and_then(Value::as_array)
                .ok_or("xref rows")?
                .iter()
                .cloned(),
        );
    }
    verify_xrefs(&all)?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    merge(&recording, directory)?;
    let ceiling = (maximum * 115).div_ceil(100);
    writeln!(
        std::io::stderr(),
        "CELL_LINE_XREF_WALK_MAX {maximum} CEILING {ceiling}"
    )
    .map_err(|e| e.to_string())?;
    if stub.finish()?.len() != 2 {
        return Err("replayed xref walk made requests".into());
    }
    Ok(())
}

fn verify_xrefs(all: &[Value]) -> Result<(), String> {
    let raw: Value = serde_json::from_str(&fixture("portable-cellosaurus-cell-CVCL_0004.json")?)
        .map_err(|e| e.to_string())?;
    let expected = raw
        .pointer("/Cellosaurus/cell-line-list/0/xref-list")
        .and_then(Value::as_array)
        .ok_or("native xrefs")?
        .clone();
    if all != expected.as_slice() || all.len() != 1398 {
        return Err("xref walk lost order or rows".into());
    }
    let unique: std::collections::BTreeSet<_> = all.iter().map(Value::to_string).collect();
    if unique.len() != 1398 {
        return Err("xref walk duplicates".into());
    }
    Ok(())
}
