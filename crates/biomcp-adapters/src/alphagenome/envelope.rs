//! Validate complete DNASE reference and alternate envelopes before publication.
use super::{Limits, StreamDecoder, Tensor, TensorStream, proto};
use super::{envelope_budget::EnvelopeBudget, wire};
use biomcp_fetch::BadReply;
use prost::Message;

/// Bounded envelope validator. Raw messages remain in the transport recording.
/// No tensor values or labels are projected before the separately reviewed capture.
#[derive(Debug)]
pub struct Envelope {
    interval: proto::Interval,
    limits: Limits,
    tensors: TensorStream,
    seen: [bool; 2],
    active: Option<(usize, usize)>,
    budget: EnvelopeBudget,
    failed: bool,
}
impl Envelope {
    pub(super) fn new(interval: &proto::Interval, limits: Limits) -> Result<Self, BadReply> {
        Ok(Self {
            interval: interval.clone(),
            limits,
            tensors: TensorStream::new(limits).map_err(|_| BadReply)?,
            seen: [false; 2],
            active: None,
            budget: EnvelopeBudget::new(limits),
            failed: false,
        })
    }
    fn read(&mut self, bytes: &[u8]) -> Result<(), BadReply> {
        if self.failed {
            return Err(BadReply);
        }
        self.budget.message(bytes)?;
        let (tag, payload) = wire::single(bytes)?;
        let tensor = match tag {
            1 | 2 => {
                let allele = usize::try_from(tag - 1).map_err(|_| BadReply)?;
                if self.active.is_some() || self.seen.get(allele).copied() != Some(false) {
                    return Err(BadReply);
                }
                let track = track(payload)?;
                let (tensor, metadata) = self.track(track)?;
                self.active = Some((allele, metadata));
                self.tensors.descriptor(tensor).map_err(|_| BadReply)?
            }
            3 => self.tensors.chunk(payload).map_err(|_| BadReply)?,
            _ => return Err(BadReply),
        };
        if let Some(tensor) = tensor {
            self.complete(tensor)?;
        }
        Ok(())
    }
    fn track<'a>(&self, bytes: &'a [u8]) -> Result<(&'a [u8], usize), BadReply> {
        let mut tensor = None;
        let mut metadata = 0usize;
        let mut resolution = false;
        let mut interval = false;
        wire::fields(bytes, |tag, value| {
            match tag {
                1 if tensor.is_none() => tensor = Some(value.bytes()?),
                2 => {
                    // Decode one bounded metadata row at a time, never a provider-sized Vec.
                    proto::TrackMetadata::decode(value.bytes()?).map_err(|_| BadReply)?;
                    metadata = metadata.checked_add(1).ok_or(BadReply)?;
                    if metadata > self.limits.dimension {
                        return Err(BadReply);
                    }
                }
                3 if !resolution => {
                    resolution = true;
                    if !(1..=i64::MAX as u64).contains(&value.number()?) {
                        return Err(BadReply);
                    }
                }
                4 if !interval => {
                    interval = true;
                    let supplied = proto::Interval::decode(value.bytes()?).map_err(|_| BadReply)?;
                    if supplied.chromosome != self.interval.chromosome
                        || supplied.start != self.interval.start
                        || supplied.end != self.interval.end
                    {
                        return Err(BadReply);
                    }
                }
                _ => return Err(BadReply),
            }
            Ok(())
        })?;
        if metadata == 0 {
            return Err(BadReply);
        }
        Ok((tensor.ok_or(BadReply)?, metadata))
    }
    fn complete(&mut self, tensor: Tensor) -> Result<(), BadReply> {
        let (allele, metadata) = self.active.take().ok_or(BadReply)?;
        if tensor.bytes().is_empty()
            || usize::try_from(*tensor.shape().last().ok_or(BadReply)?).map_err(|_| BadReply)?
                != metadata
        {
            return Err(BadReply);
        }
        *self.seen.get_mut(allele).ok_or(BadReply)? = true;
        Ok(())
    }
}
impl StreamDecoder for Envelope {
    type Reply = ();
    fn message(&mut self, bytes: &[u8]) -> Result<(), BadReply> {
        let result = self.read(bytes);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn finish(self) -> Result<(), BadReply> {
        if self.failed || self.active.is_some() || self.seen != [true; 2] {
            return Err(BadReply);
        }
        self.tensors.finish().map_err(|_| BadReply)
    }
}
fn track(bytes: &[u8]) -> Result<&[u8], BadReply> {
    let mut output_type = None;
    let mut data = None;
    wire::fields(bytes, |tag, value| {
        match tag {
            1 if output_type.is_none() => output_type = Some(value.number()?),
            2 if data.is_none() => data = Some(value.bytes()?),
            _ => return Err(BadReply),
        }
        Ok(())
    })?;
    if output_type != Some(proto::OutputType::Dnase as u64) {
        return Err(BadReply);
    }
    data.ok_or(BadReply)
}
