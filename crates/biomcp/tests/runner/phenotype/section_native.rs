//! Native term replies stay outside Git and are admitted only by exact hash.

fn measurements(key: &str) -> Result<[usize; 2], String> {
    let values: std::collections::BTreeMap<String, [usize; 2]> = serde_saphyr::from_str(
        &crate::runner::launch::fixture_text("phenotype-section-budgets.yaml")?,
    )
    .map_err(|e| e.to_string())?;
    values
        .get(key)
        .copied()
        .ok_or_else(|| format!("unmeasured phenotype case {key}"))
}
pub(super) fn ceiling(key: &str) -> Result<usize, String> {
    let [_, comparable] = measurements(key)?;
    Ok((comparable * 115).div_ceil(100))
}
