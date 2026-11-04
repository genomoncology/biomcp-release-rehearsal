//! Schema output survives closed readers; Markdown omits an absent query.
use super::{launch::Environment, process::Process};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use std::{io::Read, process::Stdio};

pub(super) fn trials() -> Vec<Trial> {
    // Keep dynamic state, source bindings or wire checks that YAML cannot express.
    vec![transcript_trial(
        "schema-discovery/closed-output-reader",
        || closed_reader().map_err(Into::into),
    )]
}

fn closed_reader() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    // The preclosed case forces the error even when help fits a pipe buffer.
    // The second case models a reader consuming ten bytes before closing.
    for read_prefix in [false, true] {
        let (reader, writer) = std::io::pipe().map_err(|e| e.to_string())?;
        let reader = if read_prefix {
            Some(reader)
        } else {
            drop(reader);
            None
        };
        let child = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &["help".into()])
            .stdout(writer)
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        if let Some(mut reader) = reader {
            let mut prefix = [0; 10];
            reader.read_exact(&mut prefix).map_err(|e| e.to_string())?;
            drop(reader);
        }
        let (status, _, stderr) = Process::new(child)?.finish()?;
        if !status.success() || !stderr.is_empty() {
            return Err(format!(
                "closed output reader (prefix={read_prefix}): {status}; stderr={}",
                String::from_utf8_lossy(&stderr)
            ));
        }
    }
    Ok(())
}
