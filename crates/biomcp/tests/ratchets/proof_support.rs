//! Build isolated source trees for the structural witnesses.
use super::{Check, check};
use std::{fs, path::Path};

pub(super) fn tree() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let tree = tempfile::tempdir()?;
    fs::write(
        tree.path().join("Cargo.toml"),
        "[workspace]\n[workspace.dependencies]\nhttp = { package = 'ureq', version = '3' }\n",
    )?;
    fs::write(tree.path().join("deny.toml"), "[sources]\nallow-git = []\n")?;
    fs::write(
        tree.path().join("Cargo.lock"),
        "version = 4\npackage = []\n",
    )?;
    put(
        tree.path(),
        "crates/biodata/Cargo.toml",
        "[package]\nname = 'biodata'\n[lints]\nworkspace = true\n",
    )?;
    put(tree.path(), "crates/biodata/src/lib.rs", "//! Values.\n")?;
    Ok(tree)
}

pub(super) fn put(root: &Path, path: &str, text: &str) -> Check {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;
    Ok(())
}

pub(super) fn rejects(path: &str, content: &str, rule: &str) -> Check {
    let tree = tree()?;
    put(tree.path(), path, content)?;
    let failure = match check(tree.path()) {
        Ok(()) => return Err("planted violation passed".into()),
        Err(error) => error.to_string(),
    };
    assert!(
        failure.contains(path) && failure.contains(rule),
        "{failure}"
    );
    Ok(())
}

pub(super) fn check_inventory_mutations() -> Check {
    use super::source::adapter_values::validate;
    let original = include_str!("adapter_value_audit.json");
    let mut current: serde_json::Value = serde_json::from_str(original)?;
    let sites = audit_sites(&mut current)?;
    sites.retain(|entry| {
        entry
            .get("maximum")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|n| n > 1)
    });
    for entry in sites.iter_mut() {
        let entry = entry.as_object_mut().ok_or("invalid audit site")?;
        entry.insert("maximum".into(), 1.into());
        entry.insert("class".into(), "migrated".into());
    }
    validate(&current.to_string())?;
    validate("{\"sites\":[]}")?;
    let planted = serde_json::json!({"sites": [{
        "path": "crates/biomcp-adapters/src/example/details.rs",
        "site": "new pending identity",
        "maximum": 1,
        "class": "pending_audit",
        "owner": "example",
        "reason": "Planted new entry"
    }]});
    assert!(
        validate(&planted.to_string()).is_err(),
        "new entry in empty inventory passed"
    );
    let mut current: serde_json::Value = serde_json::from_str(original)?;
    if audit_sites(&mut current)?.is_empty() {
        return Ok(());
    }
    audit_site(&mut current)?.insert("site".into(), "new pending identity".into());
    assert!(
        validate(&current.to_string()).is_err(),
        "same-count replacement passed"
    );
    let mut current: serde_json::Value = serde_json::from_str(original)?;
    let maximum = audit_site(&mut current)?
        .get("maximum")
        .and_then(serde_json::Value::as_u64)
        .ok_or("missing count")?;
    audit_site(&mut current)?.insert("maximum".into(), (maximum + 1).into());
    assert!(
        validate(&current.to_string()).is_err(),
        "increased ceiling passed"
    );
    let mut current: serde_json::Value = serde_json::from_str(original)?;
    audit_site(&mut current)?.insert("owner".into(), "other_adapter".into());
    assert!(
        validate(&current.to_string()).is_err(),
        "wrong adapter owner passed"
    );
    let mut current: serde_json::Value = serde_json::from_str(original)?;
    let duplicate = audit_sites(&mut current)?
        .first()
        .ok_or("missing audit site")?
        .clone();
    audit_sites(&mut current)?.push(duplicate);
    assert!(
        validate(&current.to_string()).is_err(),
        "duplicate pending entry passed"
    );
    Ok(())
}

fn audit_sites(
    value: &mut serde_json::Value,
) -> Result<&mut Vec<serde_json::Value>, Box<dyn std::error::Error>> {
    value
        .get_mut("sites")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| "missing audit sites".into())
}
fn audit_site(
    value: &mut serde_json::Value,
) -> Result<&mut serde_json::Map<String, serde_json::Value>, Box<dyn std::error::Error>> {
    audit_sites(value)?
        .first_mut()
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| "missing audit site".into())
}

pub(super) fn check_mapped_refusal_decisions() -> Check {
    let path = "crates/biomcp-adapters/src/example/map.rs";
    rejects(
        path,
        "fn mapped() { Mapped { value, problems } }",
        "explicit refusal decision",
    )?;
    rejects(
        path,
        "fn mapped() { Mapped::<Vec<Record>> { value, problems } }",
        "explicit refusal decision",
    )?;
    // A nested record's decision cannot stand in for the enclosing mapping.
    rejects(
        path,
        "fn mapped() { Mapped { value: Record { refusals: None }, problems } }",
        "explicit refusal decision",
    )?;
    let tree = tree()?;
    put(
        tree.path(),
        path,
        "fn mapped() { Mapped { value, problems, refusals: Refusal::None } }",
    )?;
    check(tree.path())
}

/// Private stored facts do not grant native fields a default allowance.
pub(super) fn private_refusal_defaults() -> Check {
    let tree = tree()?;
    let path = "crates/biomcp-adapters/src/nci/mod.rs";
    let codec = "crate::mapped::refusal_serde";
    let attribute = format!("#[serde(default = \"{codec}::none\", with = \"{codec}\")] ");
    let valid = format!("pub struct Detail {{ {attribute} pub refusals: crate::Refusal, }}");
    put(tree.path(), path, &valid)?;
    super::source::check(tree.path())?;
    for planted in [
        valid.replace("crate::Refusal", "bool"),
        valid.replace("refusals: crate::Refusal", "data: Vec<String>"),
        valid.replace("refusals: crate::Refusal", "data: crate::Refusal"),
        valid.replace("Detail", "Native"),
        valid.replace(
            "with = \"crate::mapped::refusal_serde\"",
            "with = \"other\"",
        ),
        valid.replace(", with = \"crate::mapped::refusal_serde\"", ""),
        valid.replace("::none\"", "::arbitrary\""),
        valid.replace(&attribute, "#[serde(default)] "),
    ] {
        put(tree.path(), path, &planted)?;
        let failure = match super::source::check(tree.path()) {
            Ok(()) => return Err("native or unrelated default passed".into()),
            Err(error) => error.to_string(),
        };
        assert!(failure.contains("raw serde default"), "{failure}");
    }
    put(tree.path(), path, &valid)?;
    rejects(
        "crates/biomcp-adapters/src/example/details.rs",
        &valid,
        "raw serde default",
    )?;
    let alias = "crates/biomcp-adapters/src/ctgov/refusals.rs";
    let definition = "pub(super) use crate::mapped::refusal_serde::{deserialize, none, serialize};";
    put(tree.path(), alias, definition)?;
    put(
        tree.path(),
        "crates/biomcp-adapters/src/ctgov/detail.rs",
        &valid.replace(codec, "super::refusals"),
    )?;
    super::source::check(tree.path())?;
    put(
        tree.path(),
        alias,
        &definition.replace("refusal_serde", "other"),
    )?;
    let failure = match super::source::check(tree.path()) {
        Ok(()) => return Err("changed private codec alias passed".into()),
        Err(error) => error.to_string(),
    };
    assert!(failure.contains("raw serde default"), "{failure}");
    Ok(())
}

pub(super) fn independent_structural_failures() -> Check {
    let tree = tree()?;
    put(
        tree.path(),
        "crates/biomcp-adapters/src/example/raw.rs",
        &format!(
            "#[serde(default)] struct Raw;\nfn text() {{}}\n//{}\n",
            "x".repeat(161)
        ),
    )?;
    let failure = match super::source::check(tree.path()) {
        Ok(()) => return Err("independent violations passed".into()),
        Err(error) => error.to_string(),
    };
    for category in ["raw serde default", "line width", "mapping helpers belong"] {
        assert!(failure.contains(category), "missing {category}: {failure}");
    }
    Ok(())
}
