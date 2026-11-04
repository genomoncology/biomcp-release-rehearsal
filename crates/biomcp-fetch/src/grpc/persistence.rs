//! Drive capture and replay through one checked stream finalizer.
use super::{GrpcEndpoint, Limits, StreamDecoder, rpc, spool::Spool};
use crate::{
    Leg, Mode, RecordError, RequestError,
    prepared::PostMetadata,
    record::rpc::{Protocol, Rpc},
};
use biomcp_types::Reason;
use sha2::{Digest, Sha256};
use std::time::Duration;

pub(super) struct Identity {
    pub(super) key: [u8; 32],
    pub(super) url: String,
    post: PostMetadata,
}
impl Identity {
    pub(super) fn new(
        endpoint: &GrpcEndpoint,
        request: &[u8],
        limits: Limits,
    ) -> Result<Self, RequestError> {
        let (uri, path) = rpc::checked(endpoint.spec.base, endpoint.path, limits.metadata_bytes)?;
        endpoint.spec.admit(endpoint.spec.base, false)?;
        let authority = uri.authority().ok_or(crate::InputError::Path)?.as_str();
        if endpoint.schema.len() > limits.metadata_bytes
            || !endpoint.schema.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(crate::InputError::Path.into());
        }
        let mut hash = Sha256::new();
        for value in [
            b"grpc/1".as_slice(),
            authority.as_bytes(),
            path.as_str().as_bytes(),
            endpoint.schema.as_bytes(),
            request,
        ] {
            hash.update((value.len() as u64).to_be_bytes());
            hash.update(value);
        }
        Ok(Self {
            key: hash.finalize().into(),
            url: format!("https://{authority}{path}"),
            post: PostMetadata {
                request_content_type: format!("application/grpc; schema={}", endpoint.schema),
                request_body_sha256: Sha256::digest(request)
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            },
        })
    }
}
impl Leg<'_> {
    pub(super) fn rpc_exchange<D: StreamDecoder>(
        &self,
        endpoint: &GrpcEndpoint,
        request: &[u8],
        identity: &Identity,
        mut decoder: D,
        limits: Limits,
    ) -> Result<(D::Reply, Spool), RequestError> {
        let mut spool = Spool::new(limits)?;
        let mut end = rpc::NetworkEnd::default();
        let mut local = None;
        let result = if matches!(self.fetch.mode, Mode::Replay { .. }) {
            self.rpc_replay(identity, limits, &mut spool, &mut decoder)
        } else {
            self.rpc_network(
                endpoint,
                request,
                |bytes, delay| {
                    self.rpc_contain(bytes).map_err(|_| crate::BadReply)?;
                    let delay = crate::record::millis(delay).map_err(|error| {
                        local = Some(error);
                        crate::BadReply
                    })?;
                    spool.push(bytes, delay).map_err(|error| {
                        local = Some(error);
                        crate::BadReply
                    })?;
                    decoder.message(bytes)
                },
                true,
                limits,
                &mut end,
            )
            .and_then(|()| {
                if end.terminal == Some(0) {
                    Ok(())
                } else {
                    Err(self.failure(Reason::BadReply, None).into())
                }
            })
        };
        let result = result.and_then(|()| {
            decoder
                .finish()
                .map_err(|_| self.failure(Reason::BadReply, None).into())
        });
        let result = if self.budget().expired() {
            Err(self.failure(Reason::DeadlineExceeded, None).into())
        } else {
            result
        };
        let result = if let Some(error) = local {
            Err(error.into())
        } else {
            result
        };
        let terminal = end.terminal;
        if matches!(self.fetch.mode, Mode::Record { .. }) {
            let interruption = match &result {
                Err(RequestError::Source(error)) if terminal.is_none() || terminal == Some(0) => {
                    Some(error.reason)
                }
                Err(_) if terminal.is_none() || terminal == Some(0) => Some(Reason::BadReply),
                _ => None,
            };
            self.fetch.recordings.append_rpc(
                self.spec.source,
                &identity.key,
                &identity.url,
                &identity.post,
                &mut spool,
                Rpc {
                    protocol: Protocol::GrpcV1,
                    messages: Vec::new(),
                    terminal,
                    interruption,
                    delay_ms: crate::record::millis(end.delay)?,
                },
            )?;
        }
        result.map(|reply| (reply, spool))
    }
    fn rpc_replay<D: StreamDecoder>(
        &self,
        identity: &Identity,
        limits: Limits,
        spool: &mut Spool,
        decoder: &mut D,
    ) -> Result<(), RequestError> {
        let _permit = self
            .acquire_quota()
            .map_err(|delay| self.quota_failure(delay))?
            .ok_or_else(|| self.failure(Reason::DeadlineExceeded, None))?;
        let exchange = self
            .fetch
            .recordings
            .replay(
                self.spec.source,
                &identity.key,
                &identity.url,
                Some(&identity.post).into(),
                limits.wire_bytes as u64,
            )?
            .ok_or(RecordError::Missing)?;
        let rpc = exchange.rpc.ok_or(RecordError::Malformed)?;
        if rpc.messages.len() > limits.messages {
            return Err(RecordError::Limit.into());
        }
        for message in &rpc.messages {
            if !self
                .budget()
                .advance(Duration::from_millis(message.delay_ms))
            {
                return Err(self.failure(Reason::DeadlineExceeded, None).into());
            }
            let bytes = self.fetch.recordings.rpc_message(
                self.spec.source,
                &identity.key,
                message,
                limits.message_bytes as u64,
            )?;
            self.rpc_contain(&bytes)?;
            spool.push(&bytes, message.delay_ms)?;
            decoder
                .message(&bytes)
                .map_err(|_| self.failure(Reason::BadReply, None))?;
            if self.budget().expired() {
                return Err(self.failure(Reason::DeadlineExceeded, None).into());
            }
        }
        if !self.budget().advance(Duration::from_millis(rpc.delay_ms)) {
            return Err(self.failure(Reason::DeadlineExceeded, None).into());
        }
        if let Some(reason) = rpc.interruption {
            return Err(self.failure(reason, None).into());
        }
        let code = rpc.terminal.ok_or(RecordError::Malformed)?;
        if code != 0 {
            return Err(self
                .failure(super::rpc_call::code_reason(code), None)
                .into());
        }
        Ok(())
    }
}
