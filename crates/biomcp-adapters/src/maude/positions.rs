//! Native row positions retained independently of public report values.
use crate::NativePositions;
use serde::{Deserialize, Serialize};
/// Rejected native rows keep their positions across source snapshot reloads.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Positions {
    /// Native devices positions when some rows were rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<NativePositions>,
    /// Native narrative positions when some rows were rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative: Option<NativePositions>,
}
impl Positions {
    /// Whether the native arrays retain their original dense positions.
    pub fn is_empty(&self) -> bool {
        self.devices.is_none() && self.narrative.is_none()
    }
    /// Check stored positions against their admitted report rows.
    pub fn check(&self, report: &biodata::MaudeReport) -> bool {
        self.devices
            .as_ref()
            .is_none_or(|p| p.check(report.devices.len()))
            && self
                .narrative
                .as_ref()
                .is_none_or(|p| p.check(report.narrative.len()))
    }
}
/// Checked report values and their unchanged native row positions.
#[derive(Debug)]
pub struct Report {
    /// Admitted native report values.
    pub data: biodata::MaudeReport,
    /// Positions omitted from the admitted row arrays.
    pub positions: Positions,
}
