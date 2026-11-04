//! Housekeeping selects a stable executable without trusting a different installation.
use super::{Check, Environment, success};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::Value;
use std::{fs, path::Path};

pub(super) fn trials() -> Vec<Trial> {
    let mut trials = vec![transcript_trial(
        "housekeeping/config-path-fallbacks",
        || fallbacks().map_err(|error| error.to_string().into()),
    )];
    if cfg!(unix) {
        trials.push(transcript_trial(
            "housekeeping/config-survives-symlink-upgrade",
            || upgrade().map_err(|error| error.to_string().into()),
        ));
    }
    trials
}

fn configuration(
    env: &Environment,
    program: &Path,
    path: Option<&Path>,
) -> Check<std::path::PathBuf> {
    let args = vec!["mcp-config".into(), "--offline".into()];
    let mut command = env.command(program, &args);
    if let Some(path) = path {
        command.env("PATH", path);
    } else {
        command.env_remove("PATH");
    }
    let output = command.output()?;
    let value: Value = serde_json::from_str(success(&output)?)?;
    Ok(value
        .pointer("/mcpServers/biomcp/command")
        .and_then(Value::as_str)
        .ok_or("missing executable")?
        .into())
}

fn fallbacks() -> Check {
    let env = Environment::new()?;
    let command = env.command(env!("CARGO_BIN_EXE_biomcp"), &[]);
    let executable = Path::new(command.get_program()).canonicalize()?;
    let bin = env.root().join("bin");
    fs::create_dir(&bin)?;
    for path in [None, Some(bin.as_path())] {
        let actual = configuration(&env, &executable, path)?;
        if !actual.is_absolute() || !same_file::is_same_file(&actual, &executable)? {
            return Err(format!(
                "missing PATH entry did not retain the running executable: PATH={path:?}, actual={actual:?}, expected={executable:?}"
            ).into());
        }
    }
    // A second installation can have identical bytes while naming a different file.
    fs::copy(
        &executable,
        bin.join(format!("biomcp{}", std::env::consts::EXE_SUFFIX)),
    )?;
    let actual = configuration(&env, &executable, Some(&bin))?;
    if !actual.is_absolute() || !same_file::is_same_file(&actual, &executable)? {
        return Err("a different installation replaced the running executable".into());
    }
    Ok(())
}

#[cfg(unix)]
fn upgrade() -> Check {
    let env = Environment::new()?;
    let command = env.command(env!("CARGO_BIN_EXE_biomcp"), &[]);
    let source = Path::new(command.get_program());
    let bin = env.root().join("bin");
    fs::create_dir(&bin)?;
    let stable = bin.join("biomcp");
    for version in ["1.0.0", "1.0.1"] {
        let directory = env.root().join("Cellar/biomcp").join(version).join("bin");
        fs::create_dir_all(&directory)?;
        let executable = directory.join("biomcp");
        fs::copy(source, &executable)?;
        if stable.exists() {
            fs::remove_file(&stable)?;
        }
        std::os::unix::fs::symlink(&executable, &stable)?;
        if configuration(&env, &stable, Some(&bin))? != stable {
            return Err(format!("version {version} did not retain the stable bin path").into());
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn upgrade() -> Check {
    Ok(())
}
