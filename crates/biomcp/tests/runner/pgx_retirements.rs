pub(super) fn bounded(name: &str, value: &serde_json::Value) -> Result<(), String> {
    use std::io::Write;
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    writeln!(std::io::stdout(), "{name}: {bytes}").map_err(|e| e.to_string())?;
    let measurements: std::collections::BTreeMap<String, usize> = serde_saphyr::from_str(
        &crate::runner::launch::fixture_text("pgx-command-budgets.yaml")?,
    )
    .map_err(|error| error.to_string())?;
    let baseline = measurements
        .get(name)
        .ok_or_else(|| format!("missing PGx output measurement: {name}"))?;
    if bytes > (baseline * 115).div_ceil(100) {
        return Err(format!("{name}: output exceeds measured ceiling"));
    }
    Ok(())
}
