//! Accepted mutation facts for exact query-layer matching.
use biodata::Text;
/// Checked source mutation fields after admission and optional-value mapping.
#[derive(Debug)]
pub struct Mutation {
    /// Source sample identity, used only for page-local distinct counting.
    pub sample_id: Text,
    /// Source chromosome.
    pub chr: Option<Text>,
    /// Source one-based inclusive start.
    pub start_position: Option<u64>,
    /// Source one-based inclusive end.
    pub end_position: Option<u64>,
    /// Source plus-strand reference.
    pub reference_allele: Option<Text>,
    /// Source plus-strand alternate.
    pub variant_allele: Option<Text>,
    /// Source assembly; missing never means a default.
    pub ncbi_build: Option<Text>,
}
pub(super) fn mutation(row: super::raw::Mutation) -> Mutation {
    Mutation {
        sample_id: row.sample_id,
        chr: row.chr,
        start_position: row.start_position,
        end_position: row.end_position,
        reference_allele: row.reference_allele,
        variant_allele: row.variant_allele,
        ncbi_build: row.ncbi_build,
    }
}
