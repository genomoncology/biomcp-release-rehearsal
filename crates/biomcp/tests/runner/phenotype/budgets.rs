//! Named full-envelope phenotype phases; CLI includes its newline.
use serde_json::Value;
use std::io::Write;
pub(super) fn check(value: &Value, phase: &str, surface: &str) -> Result<(), String> {
    let observed =
        serde_json::to_vec(value).map_err(|e| e.to_string())?.len() + usize::from(surface == "cli");
    bytes(observed, phase, surface)
}

pub(super) fn bytes(observed: usize, phase: &str, surface: &str) -> Result<(), String> {
    let bytes = observed;
    writeln!(
        std::io::stderr(),
        "PHENOTYPE_MEASURE {phase} [{surface}] {bytes}"
    )
    .map_err(|e| e.to_string())?;
    let max_bytes = ceiling(phase, surface)?;
    let key = format!("{phase} [{surface}]");
    if bytes > max_bytes {
        return Err(format!(
            "phenotype {key}: {bytes} exceeds max_bytes={max_bytes}"
        ));
    }
    Ok(())
}

fn ceiling(phase: &str, surface: &str) -> Result<usize, String> {
    if (phase, surface) == ("mod/help/invoke1", "cli") {
        return super::super::output_limit("cli-help");
    }
    let measurements: std::collections::BTreeMap<String, usize> = serde_saphyr::from_str(
        &crate::runner::launch::fixture_text("phenotype-command-budgets.yaml")?,
    )
    .map_err(|e| e.to_string())?;
    measurements
        .get(&format!("{phase} [{surface}]"))
        .map(|n| (n * 115).div_ceil(100))
        .ok_or("missing phenotype phase".into())
}
