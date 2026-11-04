//! Response-only fixture edits run through the ordinary command case machinery.
use serde_json::{Value, json};
use std::{fs, path::Path, sync::Arc};
tool_only! {
type Check = Result<(), Box<dyn std::error::Error>>;
pub(super) fn case(
    capture: &str,
    args: &[&str],
    change: impl FnMut(&str, &mut Value),
    expect: Value,
    exit: i32,
) -> Check {
    checked_case(capture, args, change, expect, exit).map(|_| ())
}
}

pub(super) fn checked_case(
    capture: &str,
    args: &[&str],
    change: impl FnMut(&str, &mut Value),
    expect: Value,
    exit: i32,
) -> Result<Value, Box<dyn std::error::Error>> {
    checked_steps(
        capture,
        change,
        json!([{"run": args, "expect": expect, "exit": exit}]),
    )
}

pub(super) fn checked_steps(
    capture: &str,
    mut change: impl FnMut(&str, &mut Value),
    steps: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let directory = Arc::new(tempfile::tempdir()?);
    let recordings = directory.path().canonicalize()?.join("recordings");
    copy_capture(capture, &recordings, &mut change)?;
    let path = directory.path().join("case.yaml");
    fs::write(
        &path,
        serde_json::to_vec(&json!({
            "feature": "gene-identity",
            "story": "A checked command keeps valid source facts and reports malformed siblings",
            "replay": recordings,
            "steps": steps
        }))?,
    )?;
    Ok(super::run_case_owned(
        &path,
        super::schema::Registry::embedded()?.as_ref(),
        Some(Arc::clone(&directory)),
    )?)
}

pub(super) use super::capture_support::copy_capture;

/// Acquisition is synthetic cache setup, never a claim about a retained capture.
pub(super) const CACHE_ACQUIRED: i64 = 42;
pub(super) use super::recording_support::cache_acquisition;

pub(super) fn cache_capture(capture: &str, target: &Path, acquired: i64) -> Result<(), String> {
    copy_capture(capture, target, |_, record| {
        cache_acquisition(record, acquired)
    })
    .map_err(|e| e.to_string())
}

pub(super) fn replay(
    env: &super::launch::Environment,
    capture: Option<&str>,
    acquired: Option<i64>,
) -> Result<Option<std::path::PathBuf>, String> {
    let Some(capture) = capture else {
        return Ok(None);
    };
    let Some(acquired) = acquired else {
        return Ok(Some(
            super::launch::fixtures().join("replies").join(capture),
        ));
    };
    let target = env.root().join("cache-acquisition-recordings");
    cache_capture(capture, &target, acquired)?;
    Ok(Some(target))
}
