use super::Check;
use std::{fs, path::Path};
use toml::Value;

// Runtime dependencies. The binary alone also parses case examples at build time.
// Test tooling has no product dependency edge.
const LAYERS: &[(&str, &[&str])] = &[
    ("biodata", &["serde", "winnow", "schemars"]),
    ("biomcp-types", &["biodata", "serde", "schemars"]),
    (
        "biomcp-fetch",
        &[
            "biomcp-types",
            "ureq",
            "url",
            "idna_adapter",
            "rustls",
            "ring", // Authenticated encryption for portable hosted trial continuations.
            "rustls-platform-verifier",
            "rustls-webpki",
            "tonic",
            "h2",
            "hyper",
            "tokio",
            "tokio-rustls",
            "base64",
            "prost",
            "zstd",
            "httpdate",
            "tempfile",
            "rusqlite",
            "serde",
            "serde_json",
            "quick-xml",
            "csv",
            "flate2",
            "sha2",
            "thiserror",
        ],
    ),
    (
        "biomcp-adapters",
        &[
            "prost",
            "biodata",
            "biomcp-types",
            "biomcp-fetch",
            "serde",
            "serde_json", // Redact native article text before transport persists it.
            "serde_ignored",
            "quick-xml",
            "thiserror",
            "calamine",
            "lopdf",
            "csv",
            "zip",
        ],
    ),
    (
        "biomcp-query",
        &[
            "biodata",
            "biomcp-types",
            "biomcp-fetch",
            "biomcp-adapters",
            "rusqlite",
            "serde",
            "serde_json",
            "sha2",
            "thiserror",
        ],
    ),
    (
        "biomcp",
        &[
            "biodata",
            "biomcp-types",
            "biomcp-fetch",
            "biomcp-query",
            "clap",
            "shlex",
            "sha2",
            "tempfile",
            "httparse",
            "ctrlc",
            "rpassword",
            "tempfile",
            "base64",
            "serde",
            "serde_json",
            "quick-xml",
        ],
    ),
];

fn read(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

pub(super) fn check(root: &Path) -> Check {
    let workspace = read(&root.join("Cargo.toml"))?;
    super::dependency_sources::check(root, &workspace)?;
    for (name, allowed) in LAYERS {
        let path = root.join("crates").join(name).join("Cargo.toml");
        if !path.exists() {
            continue;
        }
        let manifest = read(&path)?;
        if manifest
            .get("lints")
            .and_then(|v| v.get("workspace"))
            .and_then(Value::as_bool)
            != Some(true)
        {
            return Err(format!("{}: workspace lint inheritance required", path.display()).into());
        }
        let build_only: &[&str] = if *name == "biomcp" {
            &["serde-saphyr"]
        } else {
            &[]
        };
        check_tables(&manifest, &workspace, allowed, build_only, &path)?;
        if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
            for target in targets.values() {
                check_tables(target, &workspace, allowed, build_only, &path)?;
            }
        }
    }
    Ok(())
}

fn check_tables(
    manifest: &Value,
    workspace: &Value,
    allowed: &[&str],
    build_only: &[&str],
    path: &Path,
) -> Check {
    for kind in ["dependencies", "build-dependencies"] {
        if let Some(deps) = manifest.get(kind).and_then(Value::as_table) {
            for (alias, spec) in deps {
                let package = package(alias, spec, workspace)?;
                if !(allowed.contains(&package)
                    || kind == "build-dependencies" && build_only.contains(&package))
                {
                    return Err(format!(
                        "{}: forbidden {kind} dependency {alias} ({package})",
                        path.display()
                    )
                    .into());
                }
            }
        }
    }
    Ok(())
}

pub(super) fn package<'a>(
    alias: &'a str,
    spec: &'a Value,
    workspace: &'a Value,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let spec = if spec.get("workspace").and_then(Value::as_bool) == Some(true) {
        workspace
            .get("workspace")
            .and_then(|v| v.get("dependencies"))
            .and_then(|v| v.get(alias))
            .ok_or_else(|| format!("workspace dependency {alias} is missing"))?
    } else {
        spec
    };
    Ok(spec.get("package").and_then(Value::as_str).unwrap_or(alias))
}

pub(super) fn sqlite_aliases(root: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = root.join("crates/biomcp-query/Cargo.toml");
    let mut aliases = vec!["rusqlite".to_owned()];
    if path.exists() {
        let workspace = read(&root.join("Cargo.toml"))?;
        let manifest = read(&path)?;
        collect_sqlite(&manifest, &workspace, &mut aliases)?;
        if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
            for target in targets.values() {
                collect_sqlite(target, &workspace, &mut aliases)?;
            }
        }
    }
    Ok(aliases)
}

fn collect_sqlite(manifest: &Value, workspace: &Value, aliases: &mut Vec<String>) -> Check {
    for kind in ["dependencies", "build-dependencies"] {
        if let Some(deps) = manifest.get(kind).and_then(Value::as_table) {
            for (alias, spec) in deps {
                if package(alias, spec, workspace)? == "rusqlite" {
                    aliases.push(alias.replace('-', "_"));
                }
            }
        }
    }
    Ok(())
}
