//! Admit only a complete checked RPC into the ordinary reply cache.
use super::{GrpcEndpoint, Limits, persistence::Identity, spool};
use crate::{
    BadReply, Leg, Mode, RequestError,
    cache::{Entry, now},
};
use biomcp_types::Reason;
const FORMAT: &str = "application/biomcp-grpc-v1";
const TERMINAL: &[u8] = b"grpc/1\0\0";

/// A source decoder checks ordering, exact counts, tensor bounds and request identity.
/// The transport calls the same decoder for live, cache and replay messages.
/// Implementations must bound retained results and keep partial results private.
pub trait StreamDecoder {
    /// The fully checked source reply, exposed only after successful terminal status.
    type Reply;
    /// Check one bounded protobuf message using the supplied tensor codec where appropriate.
    fn message(&mut self, message: &[u8]) -> Result<(), BadReply>;
    /// Refuse incomplete streams and mismatched source identities before publication.
    fn finish(self) -> Result<Self::Reply, BadReply>;
}
impl Leg<'_> {
    /// Retrieve a complete checked stream without connecting in replay or offline mode.
    /// The factory creates independent decoder state when an invalid cache entry is discarded.
    pub fn grpc_checked<D: StreamDecoder>(
        &self,
        endpoint: &GrpcEndpoint,
        request: &[u8],
        decoder: impl Fn(Limits) -> Result<D, BadReply>,
    ) -> Result<D::Reply, RequestError> {
        self.check_source()?;
        if !std::ptr::eq(self.spec, endpoint.spec) || endpoint.schema.is_empty() {
            return Err(self.failure(Reason::Rejected, None).into());
        }
        let mut limits = endpoint.limits(self.fetch.config.grpc_limits);
        limits.wire_bytes = limits
            .wire_bytes
            .min(usize::try_from(self.fetch.config.max_body).unwrap_or(usize::MAX));
        limits.message_bytes = limits
            .message_bytes
            .min(limits.wire_bytes.saturating_sub(super::FRAME_BYTES));
        limits
            .check_request(request)
            .map_err(|_| crate::InputError::BodyLimit)?;
        self.rpc_contain(request)?;
        let mut identity = Identity::new(endpoint, request, limits)?;
        identity.key = self.licensed_identity(identity.key);
        if self.budget().expired() {
            return Err(self.failure(Reason::DeadlineExceeded, None).into());
        }
        let cache = self
            .fetch
            .cache
            .as_ref()
            .filter(|_| endpoint.fresh.seconds().is_some());
        if let Some(cache) =
            cache.filter(|_| !self.fresh && !matches!(self.fetch.mode, Mode::Record { .. }))
            && let Some(reply) = self.rpc_cached(cache, &identity, limits, &decoder)?
        {
            return Ok(reply);
        }
        if self.fetch.mode == Mode::Offline {
            return Err(self.failure(Reason::Offline, None).into());
        }
        let codec = decoder(limits).map_err(|_| self.failure(Reason::BadReply, None))?;
        let (reply, mut spool) = self.rpc_exchange(endpoint, request, &identity, codec, limits)?;
        if let Some(cache) = cache {
            let mut bytes = TERMINAL.to_vec();
            bytes.extend_from_slice(&spool.bytes()?);
            self.rpc_cache_result(cache.write(
                &identity.key,
                self.spec.source,
                &identity.url,
                &Entry::new(
                    200,
                    bytes,
                    endpoint.fresh.seconds().unwrap_or_default(),
                    FORMAT,
                ),
                self.budget(),
            ));
        }
        if self.budget().expired() {
            return Err(self.failure(Reason::DeadlineExceeded, None).into());
        }
        Ok(reply)
    }
    fn rpc_cached<D: StreamDecoder>(
        &self,
        cache: &crate::cache::Cache,
        identity: &Identity,
        limits: Limits,
        decoder: &impl Fn(Limits) -> Result<D, BadReply>,
    ) -> Result<Option<D::Reply>, RequestError> {
        let max_bytes = limits
            .wire_bytes
            .saturating_add(limits.messages.saturating_mul(16))
            .saturating_add(TERMINAL.len()) as u64;
        match cache.read(
            &identity.key,
            max_bytes,
            self.budget(),
            !matches!(self.fetch.mode, Mode::Offline | Mode::Replay { .. }),
        ) {
            Ok(Some(entry))
                if entry.content_type.as_deref() == Some(FORMAT)
                    && entry.status == 200
                    && (entry.fresh_until > now() || self.fetch.mode == Mode::Offline) =>
            {
                let mut codec =
                    decoder(limits).map_err(|_| self.failure(Reason::BadReply, None))?;
                let reply = entry
                    .body
                    .strip_prefix(TERMINAL)
                    .ok_or(BadReply)
                    .and_then(|body| {
                        spool::frames(body, limits, |bytes| {
                            if self.budget().expired() {
                                return Err(BadReply);
                            }
                            self.rpc_contain(bytes).map_err(|_| BadReply)?;
                            codec.message(bytes)
                        })?;
                        codec.finish()
                    });
                if self.budget().expired() {
                    return Err(self.failure(Reason::DeadlineExceeded, None).into());
                }
                if let Ok(reply) = reply {
                    if entry.fresh_until <= now() {
                        self.report
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .stale = true;
                    }
                    return Ok(Some(reply));
                }
                self.rpc_cache_result(cache.remove(&identity.key, self.budget()));
            }
            Ok(_) => {}
            Err(crate::CacheError::Busy) => self.rpc_cache_result(Err(crate::CacheError::Busy)),
            Err(error) => return Err(error.into()),
        }
        Ok(None)
    }
    fn rpc_cache_result(&self, result: Result<(), crate::CacheError>) {
        if let Err(error) = result {
            let mut report = self
                .report
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !report.cache_problems.contains(&error) {
                report.cache_problems.push(error);
            }
        }
    }
    pub(super) fn rpc_contain(&self, bytes: &[u8]) -> Result<(), RequestError> {
        let secret = match self.spec.key {
            crate::KeyUse::None => None,
            crate::KeyUse::Required(c) | crate::KeyUse::Optional(c) => {
                self.fetch.secrets.get(c.secret)
            }
        };
        if secret
            .filter(|s| !s.is_empty())
            .is_some_and(|s| bytes.windows(s.len()).any(|v| v == s.as_bytes()))
        {
            return Err(self.failure(Reason::BadReply, None).into());
        }
        Ok(())
    }
}
