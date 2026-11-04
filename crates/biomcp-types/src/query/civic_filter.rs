//! Native CIViC evidence selectors shared by drug and variant queries.
use serde::{Deserialize, Serialize};

/// Restrict evidence at CIViC before retrieving a page.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct CivicEvidenceFilter {
    /// Native CIViC evidence levels; repeat to include several levels.
    #[serde(default, skip_serializing_if = "<[ _ ]>::is_empty")]
    pub evidence_level: Box<[CivicEvidenceLevel]>,
    /// Native CIViC evidence status; omitted preserves the source's existing default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_status: Option<CivicEvidenceStatus>,
}
impl CivicEvidenceFilter {
    /// Whether the caller selected any evidence restriction.
    pub fn is_empty(&self) -> bool {
        self.evidence_level.is_empty() && self.evidence_status.is_none()
    }
    /// Distinct levels in native A through E order, or the unrestricted stream.
    pub fn levels(&self) -> Vec<Option<CivicEvidenceLevel>> {
        if self.evidence_level.is_empty() {
            return vec![None];
        }
        [
            CivicEvidenceLevel::A,
            CivicEvidenceLevel::B,
            CivicEvidenceLevel::C,
            CivicEvidenceLevel::D,
            CivicEvidenceLevel::E,
        ]
        .into_iter()
        .filter(|level| self.evidence_level.contains(level))
        .map(Some)
        .collect()
    }
}
biodata::wire_enum! {
    /// Native CIViC evidence level.
    CivicEvidenceLevel {
        /// Validated evidence.
        A => "A",
        /// Clinical evidence.
        B => "B",
        /// Case study evidence.
        C => "C",
        /// Preclinical evidence.
        D => "D",
        /// Inferential evidence.
        E => "E",
    }
}
biodata::wire_enum! {
    /// Native CIViC evidence review status.
    CivicEvidenceStatus {
        /// Accepted evidence.
        Accepted => "accepted",
        /// Submitted evidence.
        Submitted => "submitted",
        /// Rejected evidence.
        Rejected => "rejected",
    }
}
