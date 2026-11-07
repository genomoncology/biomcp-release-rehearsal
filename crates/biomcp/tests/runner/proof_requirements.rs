//! Tool proof requirements travel with the compiled runner.
use super::launch::Environment;
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) const REPOSITORY_PYTHON: &str = "repository-python";

pub(crate) fn metadata() -> Result<String, String> {
    let rows = super::tool_trials()?
        .iter()
        .map(|trial| {
            json!({"name": trial.name(),
                   "requires_repository_python": trial.kind() == REPOSITORY_PYTHON})
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&rows).map_err(|error| error.to_string())
}

pub(super) fn trial() -> Trial {
    super::transcript::trial(
        "schema-discovery/repository-python-proof-exclusions",
        || {
            let env = Environment::new()?;
            let output = env
                .command(std::env::current_exe()?, &["--proof-metadata".into()])
                .output()?;
            if !output.status.success() {
                return Err("proof metadata command failed".into());
            }
            let rows: Vec<Value> = serde_json::from_slice(&output.stdout)?;
            let excluded = rows
                .iter()
                .filter(|row| row.get("requires_repository_python") == Some(&json!(true)))
                .filter_map(|row| row.get("name").and_then(Value::as_str))
                .collect::<Vec<_>>();
            if excluded != ["mcp-http/readiness-rejects-eof-and-hung-child"]
                || rows.len() <= excluded.len()
            {
                return Err(
                    format!("repository Python proof exclusions changed: {excluded:?}").into(),
                );
            }
            Ok(())
        },
    )
}
