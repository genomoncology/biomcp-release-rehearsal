use biodata::{DeviceRecallNumber, DiagnosticDeviceId, GtrTestId, PmaNumber, WhoProductId};
use serde::Serialize;
/// Checked source-native diagnostic identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum DiagnosticId {
    /// Complete GTR accession and version.
    Gtr(GtrTestId),
    /// WHO product listing.
    Who(WhoProductId),
    /// PMA base application.
    Pma(PmaNumber),
    /// FDA clearance number.
    Clearance(DiagnosticDeviceId),
    /// FDA device recall number.
    Recall(DeviceRecallNumber),
}
impl DiagnosticId {
    /// Unchanged wire spelling.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gtr(id) => id.as_str(),
            Self::Who(id) => id.as_str(),
            Self::Pma(id) => id.as_str(),
            Self::Clearance(id) => id.as_str(),
            Self::Recall(id) => id.as_str(),
        }
    }
}
impl std::fmt::Display for DiagnosticId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
