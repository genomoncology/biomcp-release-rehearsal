//! Account for complete protobuf envelopes before any nested provider allocation.
use super::Limits;
use biomcp_fetch::BadReply;
#[derive(Debug)]
pub(super) struct EnvelopeBudget {
    limits: Limits,
    wire: usize,
    messages: usize,
}
impl EnvelopeBudget {
    pub(super) fn new(limits: Limits) -> Self {
        Self {
            limits,
            wire: 0,
            messages: 0,
        }
    }
    pub(super) fn message(&mut self, bytes: &[u8]) -> Result<(), BadReply> {
        self.wire = self.wire.checked_add(bytes.len()).ok_or(BadReply)?;
        self.messages = self.messages.checked_add(1).ok_or(BadReply)?;
        if bytes.len() > self.limits.message_bytes
            || self.wire > self.limits.wire_bytes
            || self.messages > self.limits.messages
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
