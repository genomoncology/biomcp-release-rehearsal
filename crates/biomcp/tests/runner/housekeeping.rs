//! Housekeeping runs through the isolated installed command boundary.
#[path = "../support/housekeeping_contract.rs"]
mod contract;
#[path = "housekeeping_path.rs"]
mod executable_path;

use super::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Output};
type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<Trial> {
    // Keep dynamic state, source bindings or wire checks that YAML cannot express.
    let mut trials = executable_path::trials();
    trials.push(transcript_trial(contract::OFFLINE_PROVENANCE.0, || {
        check().map_err(|error| error.to_string().into())
    }));
    trials
}
fn run(env: &Environment, args: &[&str], empty: bool) -> Check<Output> {
    let words: Vec<_> = args.iter().map(|s| (*s).to_owned()).collect();
    let mut command = env.command(env!("CARGO_BIN_EXE_biomcp"), &words);
    let maximum = limit(args, Path::new(command.get_program()))?;
    if empty {
        for (key, _) in super::launch::ENV_PATHS {
            command.env_remove(key);
        }
    }
    let output = command.output()?;
    if output.stdout.len() > maximum {
        return Err(format!(
            "housekeeping output exceeds measured ceiling: {} > {maximum}",
            output.stdout.len()
        )
        .into());
    }
    Ok(output)
}
fn success(output: &Output) -> Check<&str> {
    if !output.status.success() {
        return Err(format!("housekeeping failed: {:?}", output).into());
    }
    Ok(std::str::from_utf8(&output.stdout)?)
}
fn check() -> Check {
    let env = Environment::new()?;
    env.config(&json!({"keys":{"unknown":"never-read"},"format":"md"}))?;
    let config = env.root().join("config/biomcp/config.json");
    fs::write(&config, b"invalid config must never be read")?;
    for empty in [false, true] {
        for args in [["version", "--offline"], ["--version", "--offline"]] {
            let output = run(&env, &args, empty)?;
            if success(&output)? != format!("biomcp {}\n", env!("CARGO_PKG_VERSION")) {
                return Err("wrong version".into());
            }
        }
        let output = run(&env, &["version", "--verbose", "--offline"], empty)?;
        let text = success(&output)?;
        for field in ["Commit: ", "Target: ", "Schemas: ", "Config: ", "Cache: "] {
            if !text.contains(field) {
                return Err(format!("missing {field}: {text}").into());
            }
        }
        if !text.contains(&format!("Commit: {}\n", env!("BIOMCP_BUILD_COMMIT")))
            || !text.contains(&format!("Target: {}\n", env!("BIOMCP_BUILD_TARGET")))
            || !text.contains(<biomcp_types::Help as biomcp_types::ResponseBody>::SCHEMA)
            || !text.contains("biomcp.error/4")
        {
            return Err("verbose build or schema provenance differs from this artifact".into());
        }
        if !empty {
            let actual = text
                .lines()
                .find_map(|line| line.strip_prefix("Config: "))
                .ok_or("missing config path")?;
            if !Path::new(actual).is_absolute() || !same_file::is_same_file(actual, &config)? {
                return Err(format!(
                    "config path does not match the settings boundary: actual={actual:?}, expected={config:?}"
                ).into());
            }
        }
        for client in [
            None,
            Some("mcp-servers"),
            Some("servers"),
            Some("json"),
            Some("claude-desktop"),
            Some("claude-code"),
            Some("cursor"),
            Some("cline"),
            Some("vscode"),
        ] {
            check_config(&env, client, empty)?;
        }
        shell_client(&env, empty)?;
        invalid_client(&env, empty)?;
    }
    env.config(&json!({"format":"md","keys":{"ncbi_api_key":"never-read"}}))?;
    invalid_client(&env, false)?;
    env.config(&json!({"format":"json"}))?;
    help(&env)?;
    if env.root().join("cache/biomcp").exists() {
        return Err("housekeeping created a cache".into());
    }
    Ok(())
}
fn check_config(env: &Environment, client: Option<&str>, empty: bool) -> Check {
    let mut args = vec!["mcp-config", "--offline"];
    if let Some(client) = client {
        args.extend(["--client", client]);
    }
    let output = run(env, &args, empty)?;
    let value: Value = serde_json::from_str(success(&output)?)?;
    let key = if matches!(client, Some("servers" | "vscode")) {
        "servers"
    } else {
        "mcpServers"
    };
    let entry = value
        .get(key)
        .and_then(|v| v.get("biomcp"))
        .ok_or("missing server")?;
    let path = entry
        .get("command")
        .and_then(Value::as_str)
        .ok_or("missing executable")?;
    let expected = env.command(env!("CARGO_BIN_EXE_biomcp"), &[]);
    if !same_file::is_same_file(path, expected.get_program())?
        || !Path::new(path).is_absolute()
        || !Path::new(path).is_file()
        || entry.get("args") != Some(&json!(["serve"]))
        || value.as_object().is_none_or(|map| map.len() != 1)
    {
        return Err(format!("invalid MCP configuration: {value}").into());
    }
    Ok(())
}

fn help(env: &Environment) -> Check {
    for (name, flag) in [("version", "--verbose"), ("mcp-config", "--client")] {
        let output = run(env, &["help", name], false)?;
        let value: Value = serde_json::from_str(success(&output)?)?;
        let commands = value
            .pointer("/result/commands")
            .and_then(Value::as_array)
            .ok_or("missing help")?;
        if commands
            .iter()
            .map(|command| command.get("command"))
            .collect::<Vec<_>>()
            != vec![Some(&json!([name]))]
            || commands
                .iter()
                .find(|command| command.get("command") == Some(&json!([name])))
                .and_then(|c| c.get("arguments"))
                .and_then(Value::as_array)
                .is_none_or(|args| !args.iter().any(|arg| arg.get("name") == Some(&json!(flag))))
        {
            return Err(format!("missing {name} {flag} help: {value}").into());
        }
    }
    Ok(())
}

fn invalid_client(env: &Environment, empty: bool) -> Check {
    let output = run(
        env,
        &["mcp-config", "--client", "unknown", "--offline"],
        empty,
    )?;
    if output.status.code() != Some(2) {
        return Err("unknown client was not a usage error".into());
    }
    let value: Value = serde_json::from_slice(&output.stdout)?;
    if value.pointer("/next/0/args") != Some(&json!(["mcp-config", "--client", "mcp-servers"])) {
        return Err("invalid client omitted the supported replacement command".into());
    }
    Ok(())
}

fn shell_client(env: &Environment, empty: bool) -> Check {
    let output = run(
        env,
        &["mcp-config", "--client", "codex", "--offline"],
        empty,
    )?;
    let words = shlex::split(success(&output)?).ok_or("invalid shell command")?;
    let expected = env.command(env!("CARGO_BIN_EXE_biomcp"), &[]);
    let [codex, mcp, add, biomcp, separator, path, serve] = words.as_slice() else {
        return Err(format!("wrong shell client configuration: {words:?}").into());
    };
    if codex != "codex"
        || mcp != "mcp"
        || add != "add"
        || biomcp != "biomcp"
        || separator != "--"
        || serve != "serve"
        || !Path::new(path).is_absolute()
        || !same_file::is_same_file(path, expected.get_program())?
    {
        return Err(format!("wrong shell client configuration: {words:?}").into());
    }
    Ok(())
}

// Measured envelopes retain fifteen percent headroom. Only encoded path growth
// raises a configuration budget when an installed executable has a longer path.
fn limit(args: &[&str], executable: &Path) -> Check<usize> {
    let (mut bytes, path) = match args {
        ["version", ..] if args.contains(&"--verbose") => {
            (super::output_limit("housekeeping-version-verbose")?, None)
        }
        ["version" | "--version", ..] => (super::output_limit("housekeeping-version")?, None),
        ["help", "version", ..] => (super::output_limit("housekeeping-help-version")?, None),
        ["help", "mcp-config", ..] => (super::output_limit("housekeeping-help-mcp-config")?, None),
        ["mcp-config", ..] if args.contains(&"unknown") => {
            (super::output_limit("housekeeping-invalid-client")?, None)
        }
        ["mcp-config", ..] if args.contains(&"codex") => (
            super::output_limit("housekeeping-shell-config")?,
            Some(true),
        ),
        ["mcp-config", ..] if args.contains(&"servers") || args.contains(&"vscode") => (
            super::output_limit("housekeeping-servers-config")?,
            Some(false),
        ),
        ["mcp-config", ..] => (super::output_limit("housekeeping-config")?, Some(false)),
        _ => return Err("housekeeping command requires a measured output budget".into()),
    };
    if let Some(shell) = path {
        let path = executable.canonicalize()?;
        let path = path.to_str().ok_or("invalid executable path")?;
        let encoded = if shell {
            shlex::try_quote(path)?.len()
        } else {
            serde_json::to_vec(path)?.len() - 2
        };
        bytes += encoded.saturating_sub(63);
    }
    Ok((bytes * 115).div_ceil(100))
}
