//! Persistent MCP key snapshots and real filesystem ownership require process/file inspection.
use super::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::Value;
use std::{io::Write, process::Stdio};
const FIRST: &str = "synthetic-registration-first";
const SECOND: &str = "synthetic-registration-second";
pub(super) fn trials() -> Vec<Trial> {
    vec![
        #[cfg(unix)]
        transcript_trial("key-registration/input-before-lock", || {
            super::key_input::check().map_err(Into::into)
        }),
        transcript_trial("local-file-persistence/local-file-modes", || {
            super::local_files::modes().map_err(Into::into)
        }),
        transcript_trial("local-file-persistence/key-file-order", || {
            super::local_files::ordered(false).map_err(Into::into)
        }),
        #[cfg(unix)]
        transcript_trial("local-file-persistence/key-file-symlink", || {
            super::local_files::ordered(true).map_err(Into::into)
        }),
        transcript_trial("key-registration/key-reload-stdio", || {
            super::key_reload::check(false, false).map_err(Into::into)
        }),
        transcript_trial("key-registration/key-reload-stdio-environment", || {
            super::key_reload::check(false, true).map_err(Into::into)
        }),
        transcript_trial("key-registration/key-reload-http", || {
            super::key_reload::check(true, true).map_err(Into::into)
        }),
    ]
}
pub(super) fn invoke(
    env: &Environment,
    args: &[&str],
    input: &str,
    exit: i32,
) -> Result<Value, String> {
    let args = args.iter().map(|s| (*s).into()).collect::<Vec<_>>();
    let mut command = env.command(env!("CARGO_BIN_EXE_biomcp"), &args);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .take()
        .ok_or("missing stdin")?
        .write_all(input.as_bytes())
        .map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    command
        .record(output.status, &output.stdout, "cli")
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) {
        return Err(format!("wrong exit: {:?}", output.status));
    }
    for marker in [FIRST, SECOND] {
        if output
            .stdout
            .windows(marker.len())
            .chain(output.stderr.windows(marker.len()))
            .any(|s| s == marker.as_bytes())
        {
            return Err("credential leaked in process output".into());
        }
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
