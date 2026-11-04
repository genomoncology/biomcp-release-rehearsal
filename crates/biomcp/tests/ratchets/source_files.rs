//! Report independent file contracts without hiding later categories.
use super::{Check, adapter_values, tokens};
use std::{collections::BTreeSet, fs, path::Path};

pub(super) fn check(root: &Path) -> Check {
    let sqlite = super::super::dependencies::sqlite_aliases(root)?;
    let mut keys = BTreeSet::new();
    let crates = root.join("crates");
    let mut checks = Vec::new();
    if !crates.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(crates)? {
        let crate_root = entry?.path();
        let shared_tests = crate_root
            .file_name()
            .is_some_and(|name| name == "biomcp-test-support");
        if shared_tests {
            let manifest: toml::Value =
                toml::from_str(&fs::read_to_string(crate_root.join("Cargo.toml"))?)?;
            if manifest
                .get("package")
                .and_then(|package| package.get("publish"))
                .and_then(toml::Value::as_bool)
                != Some(false)
            {
                return Err("biomcp-test-support must declare publish=false".into());
            }
        }
        for path in super::super::rust_files(&crate_root.join("tests"))? {
            let text = fs::read_to_string(&path)?;
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            checks.push(test(&relative, &text));
        }
        for path in super::super::rust_files(&crate_root.join("src"))? {
            let text = fs::read_to_string(&path)?;
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            // This explicit dev-dependency owner contains fixtures, so product-only
            // I/O and interior-state ownership rules do not apply. General source
            // placement and module checks still scan its src directory independently.
            checks.push(if shared_tests {
                test(&relative, &text)
            } else {
                source(root, &relative, &text, &sqlite, &mut keys)
            });
        }
    }
    super::super::diagnostics(checks)
}

fn test(path: &str, text: &str) -> Check {
    let mut checks = vec![super::tool_modes(path, text), super::width(path, text)];
    if path.starts_with("crates/biomcp/tests/")
        && path != "crates/biomcp/tests/runner/launch.rs"
        && tokens(text)
            .iter()
            .any(|token| token == "\"CARGO_MANIFEST_DIR\"")
    {
        checks.push(super::fail(
            path,
            "test fixture paths belong in shared support/fixtures.rs",
        ));
    }
    if text.lines().count() > super::FILE_LIMIT {
        checks.push(super::fail(path, "file line limit exceeded"));
    }
    super::super::diagnostics(checks)
}

fn source(
    root: &Path,
    path: &str,
    text: &str,
    sqlite: &[String],
    keys: &mut BTreeSet<String>,
) -> Check {
    let input = tokens(text);
    let mut checks = vec![
        super::width(path, text),
        super::adapter_helpers(path, &input),
        super::attributes(root, path, &input),
        super::references(root, path, text, &input, sqlite, keys),
    ];
    if !path.ends_with("_tests.rs") && !path.ends_with("native_contracts.rs") {
        checks.push(super::super::errors::signatures(text));
    }
    if path.starts_with("crates/biodata/src/") && !path.ends_with("/error.rs") {
        let declaration =
            fs::read_to_string(root.join("crates/biodata/src/error.rs")).unwrap_or_default();
        checks.push(super::super::errors::display_codes(&declaration, text));
    }
    let count = text.lines().count();
    if count > super::FILE_LIMIT
        || path.starts_with("crates/biomcp-adapters/src/")
            && count > super::ADAPTER_LIMIT
            && !adapter_values::pending(path, "file line limit", count)?
    {
        checks.push(super::fail(path, "file line limit exceeded"));
    }
    if path != "crates/biomcp-types/src/response/source.rs"
        && path != "crates/biomcp-adapters/src/registry.rs"
        && input.windows(3).any(|part|
            matches!(part, [before, name, open] if before != ">" && name == "SourceOutcome" && open == "{"))
    {
        checks.push(super::fail(path, "source outcomes require registry terms"));
    }
    super::super::diagnostics(checks)
}
