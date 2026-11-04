//! PDF saves publish complete files and preserve existing paths across refusals and failed downloads.
use super::launch::{Environment, fixtures};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
type Check = Result<(), Box<dyn std::error::Error>>;
pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    vec![
        super::transcript::trial("article-pdf/pdf/atomic-explicit-saves", || {
            saves().map_err(|e| e.to_string().into())
        }),
        super::transcript::trial("article-pdf/pdf/hostile-existing-paths", || {
            existing_paths().map_err(|e| e.to_string().into())
        }),
        super::transcript::trial("article-pdf/pdf/failed-download-leaves-nothing", || {
            failures().map_err(|e| e.to_string().into())
        }),
    ]
}
fn command(
    env: &Environment,
    replay: &Path,
    args: &[String],
    exit: i32,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut full = env.globals(Some(replay), false);
    full.extend_from_slice(args);
    let output = env.command(env!("CARGO_BIN_EXE_biomcp"), &full).output()?;
    assert_eq!(
        output.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "PDF commands must keep diagnostics out of their JSON response"
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn save_args(path: &Path) -> Vec<String> {
    vec![
        "article".into(),
        "pdf".into(),
        "41351423".into(),
        "--save".into(),
        path.to_string_lossy().into(),
    ]
}
fn saves() -> Check {
    let env = Environment::new()?;
    let replay = fixtures().join("replies/article/pdf/synthetic");
    let folder = env.root().join("downloads");
    fs::create_dir(&folder)?;
    let target = folder.join("41351423.pdf");
    let receipt = command(&env, &replay, &save_args(&folder), 0)?;
    let bytes = fs::read(&target)?;
    assert!(
        bytes.starts_with(b"%PDF-"),
        "PDF saves must publish the complete file with its exact receipt"
    );
    let sha: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        receipt.pointer("/result/file/data/path"),
        Some(&json!(target.canonicalize()?.to_string_lossy())),
        "PDF saves must publish the complete file with its exact receipt"
    );
    assert_eq!(
        receipt.pointer("/result/file/data/bytes"),
        Some(&json!(bytes.len())),
        "PDF saves must publish the complete file with its exact receipt"
    );
    assert_eq!(
        receipt.pointer("/result/file/data/sha256"),
        Some(&json!(sha)),
        "PDF saves must publish the complete file with its exact receipt"
    );
    fs::write(&target, b"keep existing bytes")?;
    let refused = command(&env, &replay, &save_args(&folder), 2)?;
    assert_eq!(
        refused.pointer("/status"),
        Some(&json!("usage")),
        "PDF saves must publish the complete file with its exact receipt"
    );
    assert_eq!(
        fs::read(&target)?,
        b"keep existing bytes",
        "PDF saves must publish the complete file with its exact receipt"
    );
    let mut args = save_args(&folder);
    args.push("--replace".into());
    command(&env, &replay, &args, 0)?;
    assert_eq!(
        fs::read(&target)?,
        bytes,
        "PDF saves must publish the complete file with its exact receipt"
    );
    let mcp_path = folder.join("stdio.pdf");
    let mcp = super::mcp::call_with_globals(
        &env,
        &save_args(&mcp_path),
        false,
        &env.globals(Some(&replay), false),
    )?;
    assert_eq!(
        mcp.pointer("/result/file/data/sha256"),
        Some(&json!(sha)),
        "PDF saves must publish the complete file with its exact receipt"
    );
    assert_eq!(
        fs::read(&mcp_path)?,
        bytes,
        "PDF saves must publish the complete file with its exact receipt"
    );
    assert_eq!(
        fs::read_dir(folder)?.count(),
        2,
        "PDF saves must publish the complete file with its exact receipt"
    );
    Ok(())
}
fn existing_paths() -> Check {
    let env = Environment::new()?;
    let replay = fixtures().join("replies/article/pdf/synthetic");
    let original = env.root().join("original.pdf");
    let target = env.root().join("target.pdf");
    fs::write(&original, b"caller-owned bytes")?;
    fs::hard_link(&original, &target)?;
    command(&env, &replay, &save_args(&target), 2)?;
    let mut args = save_args(&target);
    args.push("--replace".into());
    command(&env, &replay, &args, 0)?;
    assert!(
        fs::read(&target)?.starts_with(b"%PDF-"),
        "PDF saves must preserve existing caller files and directories"
    );
    assert_eq!(
        fs::read(&original)?,
        b"caller-owned bytes",
        "PDF saves must preserve existing caller files and directories"
    );
    fs::remove_file(&target)?;
    #[cfg(unix)]
    {
        for destination in [&original, &env.root().join("missing.pdf")] {
            std::os::unix::fs::symlink(destination, &target)?;
            command(&env, &replay, &save_args(&target), 2)?;
            command(&env, &replay, &args, 2)?;
            assert!(
                fs::symlink_metadata(&target)?.file_type().is_symlink(),
                "PDF saves must preserve existing caller files and directories"
            );
            fs::remove_file(&target)?;
        }
    }
    assert_eq!(
        fs::read(&original)?,
        b"caller-owned bytes",
        "PDF saves must preserve existing caller files and directories"
    );
    assert!(
        !env.root().join("missing.pdf").exists(),
        "PDF saves must preserve existing caller files and directories"
    );
    assert!(
        fs::read_dir(env.root())?.all(|entry| {
            entry.is_ok_and(|entry| !entry.file_name().to_string_lossy().starts_with(".tmp"))
        }),
        "PDF saves must preserve existing caller files and directories"
    );
    Ok(())
}
fn failures() -> Check {
    for name in ["not-pdf", "unavailable", "timeout"] {
        let env = Environment::new()?;
        let folder = env.root().join("downloads");
        fs::create_dir(&folder)?;
        let target = folder.join("failed.pdf");
        command(
            &env,
            &fixtures().join("replies/article/pdf").join(name),
            &save_args(&target),
            4,
        )?;
        assert_eq!(fs::read_dir(&folder)?.count(), 0, "{name} left a file");
        fs::write(&target, b"preserve on failure")?;
        let mut args = save_args(&target);
        args.push("--replace".into());
        command(
            &env,
            &fixtures().join("replies/article/pdf").join(name),
            &args,
            4,
        )?;
        assert_eq!(
            fs::read(&target)?,
            b"preserve on failure",
            "Failed PDF downloads must leave no partial output behind"
        );
        assert_eq!(
            fs::read_dir(&folder)?.count(),
            1,
            "Failed PDF downloads must leave no partial output behind"
        );
    }
    Ok(())
}
