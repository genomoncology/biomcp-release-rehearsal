//! Enforce the exact source revisions declared by the dependency policy.
use super::Check;
use std::{fs, path::Path};
use toml::Value;

pub(super) fn check(root: &Path, workspace: &Value) -> Check {
    let policy: Value = toml::from_str(&fs::read_to_string(root.join("deny.toml"))?)?;
    let lock: Value = toml::from_str(&fs::read_to_string(root.join("Cargo.lock"))?)?;
    validate(workspace, &policy, &lock)
}

fn validate(workspace: &Value, policy: &Value, lock: &Value) -> Check {
    let allowed = policy
        .get("sources")
        .and_then(|sources| sources.get("allow-git"))
        .and_then(Value::as_array)
        .ok_or("source policy requires an explicit Git allowlist")?;
    let mut sources = Vec::new();
    for value in allowed {
        let source = value.as_str().ok_or("Git source must be a URL")?;
        let (_, rev) = source
            .split_once("?rev=")
            .ok_or("Git policy requires a revision")?;
        if rev.len() != 40 || !rev.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("Git policy requires a full commit".into());
        }
        sources.push(format!("git+{source}#{rev}"));
    }
    for spec in workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
        .ok_or("workspace dependencies missing")?
        .values()
    {
        if let Some(git) = spec.get("git").and_then(Value::as_str) {
            let rev = spec
                .get("rev")
                .and_then(Value::as_str)
                .ok_or("Git dependency requires a revision")?;
            let source = format!("{git}?rev={rev}");
            if !allowed.iter().any(|value| value.as_str() == Some(&source)) {
                return Err(format!("unapproved Git dependency: {source}").into());
            }
        }
    }
    for package in lock
        .get("package")
        .and_then(Value::as_array)
        .ok_or("lock packages missing")?
    {
        if let Some(source) = package.get("source").and_then(Value::as_str)
            && source.starts_with("git+")
            && !sources.iter().any(|allowed| allowed == source)
        {
            return Err(format!("unapproved locked Git source: {source}").into());
        }
    }
    Ok(())
}

#[test]
fn another_revision_in_the_allowed_repository_is_refused() -> Check {
    let revision = "a".repeat(40);
    let url = "https://example.test/values";
    let policy: Value =
        toml::from_str(&format!("[sources]\nallow-git = ['{url}?rev={revision}']"))?;
    let workspace: Value = toml::from_str(&format!(
        "[workspace.dependencies]\nvalues = {{ git = '{url}', rev = '{revision}' }}"
    ))?;
    let lock: Value = toml::from_str(&format!(
        "[[package]]\nsource = 'git+{url}?rev={revision}#{revision}'"
    ))?;
    validate(&workspace, &policy, &lock)?;
    let empty = Value::Table(Default::default());
    for (workspace, policy, lock) in [
        (&empty, &policy, &lock),
        (&workspace, &empty, &lock),
        (&workspace, &policy, &empty),
    ] {
        assert!(validate(workspace, policy, lock).is_err());
    }
    let mut changed = workspace.clone();
    changed["workspace"]["dependencies"]["values"]["rev"] = Value::String("b".repeat(40));
    assert!(validate(&changed, &policy, &lock).is_err());
    let mut changed = lock.clone();
    changed["package"][0]["source"] =
        Value::String(format!("git+{url}?rev={revision}#{}", "b".repeat(40)));
    assert!(validate(&workspace, &policy, &changed).is_err());
    Ok(())
}
