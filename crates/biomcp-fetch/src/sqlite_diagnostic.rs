//! Opt-in proof diagnostics retain only source operation coordinates and numeric codes.
use std::{fs::File, io::Write, path::Path};

/// Internal proof-owned destination; it must already exist and is never created here.
pub const SQLITE_DIAGNOSTICS: &str = "BIOMCP_SQLITE_DIAGNOSTICS";
const MAX_BYTES: u64 = 4096;

pub(crate) fn append(path: &Path, operation: &str, error: &rusqlite::Error) {
    let Ok(mut file) = File::options().append(true).open(path) else {
        return;
    };
    let Ok(metadata) = file.metadata() else {
        return;
    };
    let code = error.sqlite_error().map(|error| error.extended_code);
    let value = serde_json::json!({"operation": operation,
        "primary_code": code.map(|code| code & 0xff), "extended_code": code});
    let Ok(mut bytes) = serde_json::to_vec(&value) else {
        return;
    };
    bytes.push(b'\n');
    if metadata.len().saturating_add(bytes.len() as u64) <= MAX_BYTES {
        // Match the existing opt-in observation sink: reporting cannot replace a result.
        let _ = file.write_all(&bytes);
    }
}

/// Failure-only note of which open step stayed busy, after how many retries and how long.
pub(crate) fn append_open_failure(path: &Path, step: &str, retries: u32, elapsed_ms: u128) {
    let Ok(mut file) = File::options().append(true).open(path) else {
        return;
    };
    let Ok(metadata) = file.metadata() else {
        return;
    };
    let value = serde_json::json!({"operation": format!("open.{step}"),
        "retries": retries, "elapsed_ms": elapsed_ms});
    let Ok(mut bytes) = serde_json::to_vec(&value) else {
        return;
    };
    bytes.push(b'\n');
    if metadata.len().saturating_add(bytes.len() as u64) <= MAX_BYTES {
        let _ = file.write_all(&bytes);
    }
}
