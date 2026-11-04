use crate::native_text::NativeText;
use biodata::Text;
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Study {
    pub study_id: Text,
    pub name: Option<crate::raw::Field<NativeText>>,
    pub description: Option<crate::raw::Field<NativeText>>,
    pub citation: Option<crate::raw::Field<NativeText>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Profile {
    pub molecular_profile_id: Text,
    pub study_id: Text,
    pub molecular_alteration_type: Text,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SampleList {
    pub sample_list_id: Text,
    pub study_id: Text,
    pub sample_count: Option<crate::raw::Field<u64>>,
}
/// Native source mutation identity and sample, without inferred equivalence.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", from = "ReceivedMutation")]
pub(super) struct Mutation {
    /// Rejected native fields on this source mutation.
    pub problems: Vec<biomcp_types::Problem>,
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
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReceivedMutation {
    /// Source sample identity, used only for page-local distinct counting.
    pub sample_id: Text,
    /// Source chromosome.
    pub chr: Option<crate::raw::Field<Text>>,
    /// Source one-based inclusive start.
    pub start_position: Option<crate::raw::Field<u64>>,
    /// Source one-based inclusive end.
    pub end_position: Option<crate::raw::Field<u64>>,
    /// Source plus-strand reference.
    pub reference_allele: Option<crate::raw::Field<Text>>,
    /// Source plus-strand alternate.
    pub variant_allele: Option<crate::raw::Field<Text>>,
    /// Source assembly; missing never means a default.
    pub ncbi_build: Option<crate::raw::Field<Text>>,
}
impl From<ReceivedMutation> for Mutation {
    fn from(row: ReceivedMutation) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Cbioportal,
            record: Some(row.sample_id.clone()),
            list: Vec::new(),
        };
        Self {
            sample_id: row.sample_id,
            chr: problems.field("chr", row.chr),
            start_position: problems.field("startPosition", row.start_position),
            end_position: problems.field("endPosition", row.end_position),
            reference_allele: problems.field("referenceAllele", row.reference_allele),
            variant_allele: problems.field("variantAllele", row.variant_allele),
            ncbi_build: problems.field("ncbiBuild", row.ncbi_build),
            problems: problems.list,
        }
    }
}
macro_rules! reply {
    ($($t:ty),+ $(,)?) => {$ (impl Reply for $t { fn check(&self) -> Result<(), BadReply> { Ok(()) } })+};
}
#[derive(Deserialize)]
#[serde(transparent)]
pub(super) struct Mutations(pub Vec<super::scope::Row<Mutation, super::scope::Mutation>>);
#[derive(Deserialize)]
#[serde(transparent)]
pub(super) struct SampleLists(pub Vec<SampleList>);
reply!(Study, Profile, SampleLists, Mutations);

impl Mutations {
    pub(super) fn identities_admitted(&self) -> bool {
        let accepted = self
            .0
            .iter()
            .filter(|row| row.value.as_ref().is_some())
            .count();
        let refusals = if accepted < self.0.len() {
            crate::Refusal::Refused
        } else {
            crate::Refusal::None
        };
        refusals.identities_admitted(accepted, self.0.len())
    }
}
