//! Owning synthetic protein case phases; CLI counts its trailing newline.
use serde_json::Value;
pub(super) fn check(value: &Value, case: &str, surface: &str) -> Result<(), String> {
    let measurements: std::collections::BTreeMap<String, usize> = serde_saphyr::from_str(
        &crate::runner::launch::fixture_text("protein-synthetic-budgets.yaml")?,
    )
    .map_err(|e| e.to_string())?;
    let key = format!("{case} [{surface}]");
    let measured = measurements
        .get(&key)
        .ok_or_else(|| format!("unmeasured protein phase: {key}"))?;
    let bytes =
        serde_json::to_vec(value).map_err(|e| e.to_string())?.len() + usize::from(surface == "cli");
    let max_bytes = (measured * 115).div_ceil(100);
    if bytes > max_bytes {
        return Err(format!(
            "protein {key}: {bytes} bytes exceeds max_bytes={max_bytes}"
        ));
    }
    Ok(())
}
