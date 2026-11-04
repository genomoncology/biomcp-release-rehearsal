//! Cargo needs local Clippy configs; their shared settings must not drift.
use super::Check;
use std::{fs, path::Path};
use toml::Value;

fn read(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

pub(super) fn check(root: &Path) -> Check {
    let base = read(&root.join("clippy-base.toml"))?;
    let workspace = read(&root.join("Cargo.toml"))?;
    if workspace
        .get("workspace")
        .and_then(|v| v.get("lints"))
        .and_then(|v| v.get("rust"))
        .and_then(|v| v.get("unreachable_pub"))
        .and_then(Value::as_str)
        != Some("deny")
    {
        return Err("workspace must deny unreachable_pub".into());
    }
    for lint in [
        "allow_attributes",
        "allow_attributes_without_reason",
        "wildcard_imports",
        "missing_assert_message",
        "fn_params_excessive_bools",
        "ref_option",
        "disallowed_types",
    ] {
        if workspace
            .get("workspace")
            .and_then(|v| v.get("lints"))
            .and_then(|v| v.get("clippy"))
            .and_then(|v| v.get(lint))
            .and_then(Value::as_str)
            != Some("deny")
        {
            return Err(format!("workspace must deny {lint}").into());
        }
    }
    let mut crates = 0;
    for entry in fs::read_dir(root.join("crates"))? {
        let path = entry?.path().join("clippy.toml");
        let config = read(&path)?;
        compare(&base, &config).map_err(|error| format!("{}: {error}", path.display()))?;
        crates += 1;
    }
    if crates == 0 {
        return Err("shared lint check found no crates".into());
    }
    Ok(())
}

fn compare(base: &Value, config: &Value) -> Check {
    let fields = base.as_table().ok_or("shared Clippy base is not a table")?;
    if fields.is_empty() {
        return Err("shared Clippy base is empty".into());
    }
    for (key, expected) in fields {
        let actual = config
            .get(key)
            .ok_or_else(|| format!("missing shared {key}"))?;
        if matches!(key.as_str(), "disallowed-types" | "disallowed-methods") {
            let actual = actual
                .as_array()
                .ok_or("shared disallowed entries must be a list")?;
            for item in expected
                .as_array()
                .ok_or("shared disallowed entries must be a list")?
            {
                if !actual.contains(item) {
                    return Err(
                        format!("shared Clippy disallowed entry drift: {key}: {item:?}").into(),
                    );
                }
            }
        } else if actual != expected {
            return Err(format!("shared Clippy setting drift: {key}").into());
        }
    }
    Ok(())
}

#[test]
fn planted_clippy_drift_is_rejected() -> Check {
    let base: Value = toml::from_str(
        "too-many-lines-threshold=80\ndisallowed-types=[{path='std::cell::Cell',reason='Reviewed state only'}]",
    )?;
    compare(&base, &base)?;
    for changed in [
        "too-many-lines-threshold=81\ndisallowed-types=[{path='std::cell::Cell',reason='Reviewed state only'}]",
        "too-many-lines-threshold=80\ndisallowed-types=[]",
        "too-many-lines-threshold=80\ndisallowed-types=[{path='std::cell::Cell',reason='Unreviewed state'}]",
    ] {
        let config = toml::from_str(changed)?;
        assert!(
            compare(&base, &config).is_err(),
            "shared settings must refuse planted drift"
        );
    }
    Ok(())
}
