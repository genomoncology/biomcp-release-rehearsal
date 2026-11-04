//! Spool bounded raw messages while a caller checks one decoded stream.
use super::Limits;
use crate::RecordError;
use std::io::{Read, Seek, SeekFrom, Write};

pub(crate) struct Spool {
    file: tempfile::NamedTempFile,
    pub(crate) messages: Vec<Message>,
    wire: usize,
    limits: Limits,
}
pub(crate) struct Message {
    offset: u64,
    length: usize,
    pub(crate) delay_ms: u64,
}
impl Spool {
    pub(crate) fn new(limits: Limits) -> Result<Self, RecordError> {
        Ok(Self {
            file: tempfile::NamedTempFile::new().map_err(RecordError::from)?,
            messages: Vec::new(),
            wire: 0,
            limits,
        })
    }
    pub(crate) fn push(&mut self, bytes: &[u8], delay_ms: u64) -> Result<(), RecordError> {
        self.wire = self
            .wire
            .checked_add(bytes.len())
            .and_then(|n| n.checked_add(super::FRAME_BYTES))
            .ok_or(RecordError::Limit)?;
        if bytes.len() > self.limits.message_bytes
            || self.wire > self.limits.wire_bytes
            || self.messages.len() >= self.limits.messages
        {
            return Err(RecordError::Limit);
        }
        let offset = self
            .file
            .as_file_mut()
            .stream_position()
            .map_err(RecordError::from)?;
        self.file
            .write_all(&delay_ms.to_be_bytes())
            .map_err(RecordError::from)?;
        self.file
            .write_all(&(bytes.len() as u64).to_be_bytes())
            .map_err(RecordError::from)?;
        self.file.write_all(bytes).map_err(RecordError::from)?;
        self.messages.push(Message {
            offset: offset + 16,
            length: bytes.len(),
            delay_ms,
        });
        Ok(())
    }
    pub(crate) fn message(&mut self, index: usize) -> Result<Vec<u8>, RecordError> {
        let value = self.messages.get(index).ok_or(RecordError::Malformed)?;
        self.file
            .as_file_mut()
            .seek(SeekFrom::Start(value.offset))
            .map_err(RecordError::from)?;
        let mut bytes = vec![0; value.length];
        self.file
            .read_exact(&mut bytes)
            .map_err(RecordError::from)?;
        Ok(bytes)
    }
    pub(crate) fn bytes(&mut self) -> Result<Vec<u8>, RecordError> {
        self.file
            .as_file_mut()
            .rewind()
            .map_err(RecordError::from)?;
        let mut bytes = Vec::new();
        self.file
            .read_to_end(&mut bytes)
            .map_err(RecordError::from)?;
        Ok(bytes)
    }
}
pub(super) fn frames(
    bytes: &[u8],
    limits: Limits,
    mut consume: impl FnMut(&[u8]) -> Result<(), crate::BadReply>,
) -> Result<(), crate::BadReply> {
    let mut rest = bytes;
    let (mut count, mut wire) = (0usize, 0usize);
    while !rest.is_empty() {
        let header = rest.get(..16).ok_or(crate::BadReply)?;
        let length = usize::try_from(u64::from_be_bytes(
            header
                .get(8..16)
                .ok_or(crate::BadReply)?
                .try_into()
                .map_err(|_| crate::BadReply)?,
        ))
        .map_err(|_| crate::BadReply)?;
        count = count.checked_add(1).ok_or(crate::BadReply)?;
        wire = wire
            .checked_add(length)
            .and_then(|n| n.checked_add(super::FRAME_BYTES))
            .ok_or(crate::BadReply)?;
        if count > limits.messages || wire > limits.wire_bytes || length > limits.message_bytes {
            return Err(crate::BadReply);
        }
        rest = rest.get(16..).ok_or(crate::BadReply)?;
        consume(rest.get(..length).ok_or(crate::BadReply)?)?;
        rest = rest.get(length..).ok_or(crate::BadReply)?;
    }
    Ok(())
}
