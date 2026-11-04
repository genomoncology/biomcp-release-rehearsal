//! Consume a tensor descriptor followed by its exact declared chunk sequence.
use super::{CodecError, Limits, preflight, proto};
use prost::Message;

/// Checked provider scalar storage type. Bytes retain the provider's little-endian layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataType(proto::DataType);
impl DataType {
    fn parse(value: i32) -> Result<Self, CodecError> {
        let dtype = proto::DataType::try_from(value).map_err(|_| CodecError::BadReply)?;
        if dtype == proto::DataType::Unspecified {
            return Err(CodecError::BadReply);
        }
        Ok(Self(dtype))
    }
    /// Scalar width in bytes, including boolean and integer storage types.
    pub fn width(self) -> usize {
        match self.0 {
            proto::DataType::Bfloat16 | proto::DataType::Float16 => 2,
            proto::DataType::Float32 | proto::DataType::Int32 | proto::DataType::Uint32 => 4,
            proto::DataType::Float64 | proto::DataType::Int64 | proto::DataType::Uint64 => 8,
            _ => 1,
        }
    }
    /// Original stable protobuf enum number.
    pub fn number(self) -> i32 {
        self.0 as i32
    }
}
/// One fully checked tensor. Stream success still requires terminal transport status.
#[derive(Debug)]
pub struct Tensor {
    shape: Vec<i32>,
    dtype: DataType,
    bytes: Vec<u8>,
}
impl Tensor {
    /// Checked tensor dimensions. Empty shape denotes a scalar.
    pub fn shape(&self) -> &[i32] {
        &self.shape
    }
    /// Checked provider scalar type.
    pub fn data_type(&self) -> DataType {
        self.dtype
    }
    /// Expanded row-major tensor data with exact shape and dtype byte length.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
#[derive(Debug)]
struct Active {
    tensor: Tensor,
    expected: usize,
    remaining: usize,
}
/// A bounded tensor sequence decoder independent of RPC method and network transport.
/// Returned tensors can be consumed immediately; the decoder retains only one tensor.
#[derive(Debug)]
pub struct TensorStream {
    limits: Limits,
    active: Option<Active>,
    wire: usize,
    messages: usize,
    expanded: usize,
    tensors: usize,
    failed: bool,
}
impl TensorStream {
    /// Validate explicit limits before accepting any provider message.
    pub fn new(limits: Limits) -> Result<Self, CodecError> {
        Ok(Self {
            limits: limits.check()?,
            active: None,
            wire: 0,
            messages: 0,
            expanded: 0,
            tensors: 0,
            failed: false,
        })
    }
    fn account(&mut self, bytes: &[u8]) -> Result<(), CodecError> {
        self.wire = self
            .wire
            .checked_add(bytes.len())
            .ok_or(CodecError::BadReply)?;
        self.messages = self.messages.checked_add(1).ok_or(CodecError::BadReply)?;
        if self.failed
            || bytes.len() > self.limits.message_bytes
            || self.wire > self.limits.wire_bytes
            || self.messages > self.limits.messages
        {
            return Err(CodecError::BadReply);
        }
        Ok(())
    }
    /// Decode one serialized Tensor descriptor, refusing a descriptor during pending chunks.
    pub fn descriptor(&mut self, bytes: &[u8]) -> Result<Option<Tensor>, CodecError> {
        let result = self.start(bytes);
        if result.is_err() {
            self.failed = true;
            self.active = None;
        }
        result
    }
    fn start(&mut self, bytes: &[u8]) -> Result<Option<Tensor>, CodecError> {
        self.account(bytes)?;
        if self.active.is_some() {
            return Err(CodecError::BadReply);
        }
        preflight::descriptor(bytes, self.limits)?;
        let tensor = proto::Tensor::decode(bytes).map_err(|_| CodecError::BadReply)?;
        let dtype = DataType::parse(tensor.data_type)?;
        let expected = tensor
            .shape
            .iter()
            .try_fold(dtype.width(), |size, dimension| {
                let dimension = usize::try_from(*dimension).map_err(|_| CodecError::BadReply)?;
                size.checked_mul(dimension).ok_or(CodecError::BadReply)
            })?;
        self.expanded = self
            .expanded
            .checked_add(expected)
            .ok_or(CodecError::BadReply)?;
        if self.expanded > self.limits.expanded_bytes {
            return Err(CodecError::BadReply);
        }
        let (remaining, inline) = match tensor.payload {
            Some(proto::tensor::Payload::ChunkCount(count)) => {
                let count = usize::try_from(count).map_err(|_| CodecError::BadReply)?;
                if count == 0 || count > self.limits.chunks {
                    return Err(CodecError::BadReply);
                }
                (count, None)
            }
            Some(proto::tensor::Payload::Array(chunk)) => (1, Some(chunk)),
            None => return Err(CodecError::BadReply),
        };
        self.active = Some(Active {
            tensor: Tensor {
                shape: tensor.shape,
                dtype,
                bytes: Vec::new(),
            },
            expected,
            remaining,
        });
        if let Some(chunk) = inline {
            self.append(chunk)
        } else {
            Ok(None)
        }
    }
    /// Decode one serialized TensorChunk under the pending descriptor.
    pub fn chunk(&mut self, bytes: &[u8]) -> Result<Option<Tensor>, CodecError> {
        let result = (|| {
            self.account(bytes)?;
            if self.active.is_none() {
                return Err(CodecError::BadReply);
            }
            preflight::chunk(bytes)?;
            self.append(proto::TensorChunk::decode(bytes).map_err(|_| CodecError::BadReply)?)
        })();
        if result.is_err() {
            self.failed = true;
            self.active = None;
        }
        result
    }
    fn append(&mut self, chunk: proto::TensorChunk) -> Result<Option<Tensor>, CodecError> {
        let active = self.active.as_mut().ok_or(CodecError::BadReply)?;
        if chunk.compression_type == 0 {
            extend(active, &chunk.data)?;
        } else if chunk.compression_type == 1 {
            biomcp_fetch::grpc::decompress(&chunk.data, self.limits.zstd_window_log, |bytes| {
                extend(active, bytes)
            })?;
        } else {
            return Err(CodecError::BadReply);
        }
        active.remaining -= 1;
        if active.remaining != 0 {
            return Ok(None);
        }
        if active.tensor.bytes.len() != active.expected {
            return Err(CodecError::BadReply);
        }
        self.tensors += 1;
        Ok(self.active.take().map(|active| active.tensor))
    }
    /// Require a nonempty, complete sequence. This does not substitute for OK RPC trailers.
    pub fn finish(self) -> Result<(), CodecError> {
        if self.failed || self.active.is_some() || self.tensors == 0 {
            return Err(CodecError::BadReply);
        }
        Ok(())
    }
}
fn extend(active: &mut Active, bytes: &[u8]) -> Result<(), CodecError> {
    let next = active
        .tensor
        .bytes
        .len()
        .checked_add(bytes.len())
        .ok_or(CodecError::BadReply)?;
    if next > active.expected {
        return Err(CodecError::BadReply);
    }
    // Exact reserve avoids geometric capacity growth beyond the checked allocation envelope.
    active
        .tensor
        .bytes
        .try_reserve_exact(bytes.len())
        .map_err(|_| CodecError::BadReply)?;
    active.tensor.bytes.extend_from_slice(bytes);
    Ok(())
}
