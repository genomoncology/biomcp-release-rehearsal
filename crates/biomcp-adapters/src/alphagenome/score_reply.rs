//! Retain complete scoring tensors for bounded native-value paging.
use super::{Tensor, proto};
use biomcp_fetch::BadReply;
use biomcp_types::{AlphaGenomeGene, AlphaGenomeTrack, AlphaGenomeVariant};
/// One native group; its ordinal alone associates it with the requested scorer.
#[derive(Debug)]
pub struct ScoreGroup {
    /// Complete checked scalar storage, including every native plane.
    pub tensor: Tensor,
    /// Native track metadata in source order.
    pub tracks: Box<[Option<AlphaGenomeTrack>]>,
    /// Native gene metadata in source order, empty for a nongene aggregate.
    pub genes: Box<[Option<AlphaGenomeGene>]>,
}
impl ScoreGroup {
    pub(super) fn checked(tensor: Tensor, metadata: Metadata) -> Result<Self, BadReply> {
        let shape = tensor.shape();
        if tensor.data_type().number() != proto::DataType::Float32 as i32
            || shape.len() != 3
            || shape.first().is_none_or(|value| {
                *value < 1
                    || usize::try_from(*value)
                        .is_ok_and(|value| value > super::score_envelope::headroom(2))
            })
            || shape.get(2).is_none_or(|value| *value < 1)
            || usize::try_from(*shape.last().ok_or(BadReply)?).map_err(|_| BadReply)?
                != metadata.tracks.len()
            || (!metadata.genes.is_empty()
                && usize::try_from(*shape.get(1).ok_or(BadReply)?).map_err(|_| BadReply)?
                    != metadata.genes.len())
            || (metadata.genes.is_empty() && shape.get(1) != Some(&1))
        {
            return Err(BadReply);
        }
        Ok(Self {
            tensor,
            tracks: metadata.tracks.into(),
            genes: metadata.genes.into(),
        })
    }
}
/// Complete source reply, published only after successful terminal transport status.
#[derive(Debug)]
pub struct ScoreReply {
    /// Actual refused native scorer metadata records.
    pub refusals: crate::Refusal,
    /// Requested variant identity matched by every admitted scalar group.
    pub variant: AlphaGenomeVariant,
    /// Complete native groups in their received order.
    pub groups: Vec<ScoreGroup>,
    /// Rejected independent metadata and affected scalar identities.
    pub problems: Vec<biomcp_types::Problem>,
}
#[derive(Debug)]
pub(super) struct Metadata {
    pub(super) refusals: crate::Refusal,
    pub(super) problems: Vec<biomcp_types::Problem>,
    pub(super) tracks: Vec<Option<AlphaGenomeTrack>>,
    pub(super) genes: Vec<Option<AlphaGenomeGene>>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            refusals: crate::Refusal::None,
            problems: Vec::new(),
            tracks: Vec::new(),
            genes: Vec::new(),
        }
    }
}
