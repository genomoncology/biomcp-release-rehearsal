//! Fixed full-envelope study command-phase ceilings from synthetic CLI/MCP proof j14953.
use serde_json::Value;

pub(super) fn check(value: &Value, phase: &str, surface: &str) -> Result<(), String> {
    let measurements: std::collections::BTreeMap<String, [usize; 2]> = serde_saphyr::from_str(
        &crate::runner::launch::fixture_text("study-command-budgets.yaml")?,
    )
    .map_err(|error| error.to_string())?;
    let [cli, mcp] = measurements
        .get(phase)
        .ok_or_else(|| format!("study phase has no output measurement: {phase}"))?;
    let measured = match surface {
        "cli" => *cli,
        "mcp" => *mcp,
        _ => return Err(format!("unknown study surface: {surface}")),
    };
    let bytes =
        serde_json::to_vec(value).map_err(|e| e.to_string())?.len() + usize::from(surface == "cli");
    if bytes > (measured * 115).div_ceil(100) {
        return Err(format!(
            "study {phase} {surface} exceeds measured output ceiling: {bytes}"
        ));
    }
    Ok(())
}
