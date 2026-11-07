use crate::Text;
use biodata::Curie;
use serde::Serialize;
/// Exact concept identity or native SEER site selector.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum DiseaseId {
    /// Ontology identity retained from checked input.
    Concept(Curie),
    /// Native site code or publisher site name, in caller spelling.
    SeerSite(Text),
}
impl DiseaseId {
    /// Unchanged wire spelling.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Concept(id) => id.as_str(),
            Self::SeerSite(id) => id.as_str(),
        }
    }
}
impl std::fmt::Display for DiseaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
