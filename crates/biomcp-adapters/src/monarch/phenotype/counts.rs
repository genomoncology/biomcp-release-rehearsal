//! Native count identity stays independent of its label and scalar metadata.
use crate::{mapped::Problems, raw::Field};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Count {
    key: biodata::Curie,
    category: biodata::Curie,
    label: Option<Field<Box<str>>>,
    count: Option<Field<u64>>,
    count_direct: Option<Field<u64>>,
    count_with_orthologs: Option<Field<u64>>,
}
impl Count {
    pub(super) fn mapped(self, problems: &mut Problems) -> biodata::PhenotypeAssociationCount {
        biodata::PhenotypeAssociationCount {
            key: self.key,
            category: self.category,
            label: problems.required_field("association_counts.label", self.label),
            count: problems.required_field("association_counts.count", self.count),
            count_direct: problems.field("association_counts.count_direct", self.count_direct),
            count_with_orthologs: problems.field(
                "association_counts.count_with_orthologs",
                self.count_with_orthologs,
            ),
        }
    }
}
