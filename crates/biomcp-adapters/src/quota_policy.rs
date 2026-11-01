//! Source-owned operator override validation and policy reference inventory.
use biomcp_fetch::{PacingOverride, Quota, SourcePacing};
use std::collections::BTreeMap;

/// One source declaration supplies defaults, ceilings and the primary policy citation.
#[derive(Debug)]
pub struct SourcePolicy {
    /// Defaults are the published ceiling or the retained conservative local ceiling.
    pub quota: &'static Quota,
    /// Check date, primary URL and numeric policy or bounded absence finding.
    pub citation: &'static str,
}
/// Refuse unknown buckets or invalid values before transport startup.
pub fn configure(input: &BTreeMap<String, PacingOverride>) -> Result<SourcePacing, String> {
    let mut pacing = SourcePacing::default();
    for (name, value) in input {
        let policy = crate::quotas::policies()
            .iter()
            .find(|p| p.quota.key().name() == name)
            .ok_or_else(|| format!("source_pacing.{name}"))?;
        pacing
            .insert(policy.quota, *value)
            .map_err(|field| format!("source_pacing.{name}.{field}"))?;
    }
    Ok(pacing)
}
