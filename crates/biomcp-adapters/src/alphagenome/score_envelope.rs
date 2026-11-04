//! Admit complete scorer groups by request ordinal without inventing native scorer labels.
use super::{Limits, StreamDecoder, TensorStream, proto};
use super::{
    envelope_budget::EnvelopeBudget,
    score_metadata,
    score_reply::{Metadata, ScoreGroup, ScoreReply},
    wire,
};
use biomcp_fetch::BadReply;
use prost::Message;
pub(super) const OBSERVED_GENE_ROWS: usize = 13;
/// Measured scoring ceilings; the same limits bind transport and complete decoder admission.
/// Request and response byte/count ceilings retain at most fifteen percent headroom.
pub const SCORE_LIMITS: Limits = Limits {
    request_bytes: headroom(65),
    message_bytes: headroom(74_988),
    wire_bytes: headroom(130_883),
    expanded_bytes: headroom(70_620),
    messages: headroom(6),
    dimension: headroom(667),
    chunks: headroom(1),
    rank: 3,
    // The measured ZSTD frame window uses the codec's next whole log2 ceiling.
    zstd_window_log: headroom(69_368).next_power_of_two().ilog2(),
    ..Limits::DEFAULT
};

/// Bounded admission for the ruled ScoreVariant response.
/// The protocol does not label scorer groups: valid complete group swaps cannot be detected.
/// The caller may associate completed groups only with their requested ordinal.
#[derive(Debug)]
pub struct ScoreEnvelope {
    variant: proto::Variant,
    limits: Limits,
    budget: EnvelopeBudget,
    tensors: TensorStream,
    expected: usize,
    complete: usize,
    active: Option<Metadata>,
    groups: Vec<ScoreGroup>,
    failed: bool,
    problems: Vec<biomcp_types::Problem>,
    refusals: crate::Refusal,
}
impl ScoreEnvelope {
    pub(super) fn new(
        variant: &proto::Variant,
        expected: usize,
        limits: Limits,
    ) -> Result<Self, BadReply> {
        let limits = limits.minimum(SCORE_LIMITS);
        Ok(Self {
            variant: variant.clone(),
            limits,
            budget: EnvelopeBudget::new(limits),
            tensors: TensorStream::new(limits).map_err(|_| BadReply)?,
            expected,
            complete: 0,
            active: None,
            groups: Vec::new(),
            failed: false,
            problems: Vec::new(),
            refusals: crate::Refusal::None,
        })
    }
    fn read(&mut self, bytes: &[u8]) -> Result<(), BadReply> {
        if self.failed {
            return Err(BadReply);
        }
        self.budget.message(bytes)?;
        let (tag, payload) = wire::single(bytes)?;
        let tensor = match tag {
            1 if self.active.is_none() && self.complete < self.expected => {
                let (tag, data) = wire::single(payload)?;
                if tag != 1 {
                    return Err(BadReply);
                }
                let (tensor, metadata) = self.data(data)?;
                self.active = Some(metadata);
                self.tensors.descriptor(tensor).map_err(|_| BadReply)?
            }
            2 if self.active.is_some() => self.tensors.chunk(payload).map_err(|_| BadReply)?,
            _ => return Err(BadReply),
        };
        if let Some(tensor) = tensor {
            let mut metadata = self.active.take().ok_or(BadReply)?;
            self.problems.append(&mut metadata.problems);
            self.refusals.include(metadata.refusals);
            self.groups.push(ScoreGroup::checked(tensor, metadata)?);
            self.complete += 1;
        }
        Ok(())
    }
    fn data<'a>(&self, bytes: &'a [u8]) -> Result<(&'a [u8], Metadata), BadReply> {
        let mut tensor = None;
        let mut metadata = None;
        wire::fields(bytes, |tag, value| {
            match tag {
                1 if tensor.is_none() => tensor = Some(value.bytes()?),
                2 if metadata.is_none() => metadata = Some(self.metadata(value.bytes()?)?),
                _ => return Err(BadReply),
            }
            Ok(())
        })?;
        Ok((tensor.ok_or(BadReply)?, metadata.ok_or(BadReply)?))
    }
    fn metadata(&self, bytes: &[u8]) -> Result<Metadata, BadReply> {
        let mut variant = false;
        let mut identity = true;
        let mut tracks = 0usize;
        let mut genes = 0usize;
        let mut metadata = Metadata::default();
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::AlphaGenome,
            record: crate::mapped::record(format!(
                "{}:{}:{}>{}",
                self.variant.chromosome,
                self.variant.position,
                self.variant.reference_bases,
                self.variant.alternate_bases
            )),
            list: Vec::new(),
        };
        wire::fields(bytes, |tag, value| {
            let value = value.bytes()?;
            match tag {
                1 if !variant => {
                    if proto::Variant::decode(value).map_err(|_| BadReply)? != self.variant {
                        problems
                            .required::<()>(&format!("scorers.{}.variant", self.complete), None);
                        identity = false;
                        metadata.refusals = crate::Refusal::Refused;
                    }
                    variant = true;
                }
                2 => {
                    tracks = tracks.checked_add(1).ok_or(BadReply)?;
                    if tracks > self.limits.dimension {
                        return Err(BadReply);
                    }
                    metadata.tracks.push(score_metadata::track(
                        proto::TrackMetadata::decode(value).map_err(|_| BadReply)?,
                        &format!("scorers.{}.tracks.{}", self.complete, tracks - 1),
                        &mut problems,
                        &mut metadata.refusals,
                    ));
                }
                3 => {
                    genes = genes.checked_add(1).ok_or(BadReply)?;
                    if genes > self.limits.dimension.min(headroom(OBSERVED_GENE_ROWS)) {
                        return Err(BadReply);
                    }
                    metadata.genes.push(score_metadata::gene(
                        proto::GeneScorerMetadata::decode(value).map_err(|_| BadReply)?,
                        &format!("scorers.{}.genes.{}", self.complete, genes - 1),
                        &mut problems,
                        &mut metadata.refusals,
                    ));
                }
                _ => return Err(BadReply),
            }
            Ok(())
        })?;
        if !variant {
            return Err(BadReply);
        }
        if !identity {
            metadata.tracks.iter_mut().for_each(|track| *track = None);
        }
        metadata.problems = problems.list;
        Ok(metadata)
    }
}
pub(super) const fn headroom(measured: usize) -> usize {
    (measured * 115).div_ceil(100)
}
impl StreamDecoder for ScoreEnvelope {
    type Reply = ScoreReply;
    fn message(&mut self, bytes: &[u8]) -> Result<(), BadReply> {
        let result = self.read(bytes);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn finish(self) -> Result<ScoreReply, BadReply> {
        if self.failed || self.active.is_some() || self.complete != self.expected {
            return Err(BadReply);
        }
        self.tensors.finish().map_err(|_| BadReply)?;
        Ok(ScoreReply {
            refusals: self.refusals,
            variant: score_metadata::variant(&self.variant)?,
            groups: self.groups,
            problems: self.problems,
        })
    }
}
