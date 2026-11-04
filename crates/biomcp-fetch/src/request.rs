use crate::read::Read;
use crate::record::{Body, Exchange};
use crate::{Arg, Endpoint, Leg, Reply, RequestError, SourceError};
use crate::{InputError, declaration::Method, prepared::Prepared};
use biomcp_types::Reason;
use std::time::{Duration, Instant};

pub(crate) struct Decoded<T> {
    pub(crate) reply: T,
    pub(crate) status: u16,
    pub(crate) body: Vec<u8>,
    pub(crate) fetched: Option<i64>,
    pub(crate) content_range: Option<String>,
    pub(crate) pagination: Option<crate::Pagination>,
}
#[derive(serde::Serialize)]
struct Wire {
    fetched: Option<i64>,
    status: u16,
    content_type: String,
    content_range: Option<String>,
    pagination: Option<crate::Pagination>,
    #[serde(skip)] // Raw and decoded body scanning stays separate from metadata.
    body: Vec<u8>,
    retry_after: Option<Duration>,
}
pub(crate) enum Failure {
    Transport(ureq::Error),
    Local(RequestError),
}

impl Leg<'_> {
    /// Probe declared reachability through the same bounded transport, quota and replay policy.
    /// Probe replies never enter the checked data cache.
    pub fn probe(&self, ep: &Endpoint, input: crate::Input<'_>) -> Result<(), RequestError> {
        let mut ep = ep.clone().fresh(crate::Freshness::Never);
        // Reachability is one observation, never a retrying data request.
        ep.idempotent = false;
        self.call(&ep, input, Read::probe()).map(|_| ())
    }

    pub(crate) fn parameters<T: Reply>(
        &self,
        ep: &Endpoint,
        params: &[(&str, Arg<'_>)],
        codec: Read<T>,
    ) -> Result<(T, crate::ReplyMetadata), RequestError> {
        self.validate(ep, false)?;
        let url = if ep.publisher {
            ep.path.to_string()
        } else {
            format!("{}{}", self.spec.base.trim_end_matches('/'), ep.path)
        };
        let (url, body) = if matches!(ep.method, Method::Get | Method::Head) {
            (
                crate::retry::canonical(&url, params, self.spec.key),
                Vec::new(),
            )
        } else {
            (
                url,
                crate::prepared::form(params, self.spec.key, self.fetch.config.max_body)?,
            )
        };
        let mut prepared = Prepared::new(url, body, ep.method, ep.idempotent);
        prepared.outgoing = params
            .iter()
            .filter_map(|(name, value)| match value {
                Arg::Contact(secret) => {
                    Some(((*name).to_owned(), crate::Secret::new(secret.value())))
                }
                _ => None,
            })
            .collect();
        self.send(ep, prepared, codec)
    }

    pub(crate) fn failure(&self, reason: Reason, status: Option<u16>) -> SourceError {
        SourceError {
            source: self.spec.source,
            reason,
            status,
            tls_cause: None,
            retry_after: None,
        }
    }

    pub(crate) fn quota_failure(&self, delay: Duration) -> SourceError {
        let mut error = self.failure(Reason::LocalQuota, None);
        error.retry_after = Some(delay);
        error
    }

    pub(crate) fn check_source(&self) -> Result<(), RequestError> {
        if self.fetch.disabled(self.spec.source) {
            return Err(SourceError::disabled(self.spec.source).into());
        }
        if let crate::KeyUse::Required(credential) | crate::KeyUse::Optional(credential) =
            self.spec.key
            && self.fetch.secrets.invalid(credential.secret)
        {
            return Err(InputError::CredentialEncoding.into());
        }
        if self.no_key {
            return Err(self.failure(Reason::NoKey, None).into());
        }
        if self.wrong_clock {
            return Err(self.failure(Reason::Rejected, None).into());
        }
        Ok(())
    }
    pub(crate) fn validate(&self, ep: &Endpoint, json_body: bool) -> Result<(), RequestError> {
        self.check_source()?;
        if !std::ptr::eq(self.spec, ep.spec) {
            return Err(self.failure(Reason::Rejected, None).into());
        }
        if (ep.method == Method::Json) != json_body {
            return Err(InputError::Method.into());
        }
        if ep.publisher {
            self.spec.admit(&ep.path, true)?;
            let uri = ep
                .path
                .parse::<ureq::http::Uri>()
                .map_err(|_| InputError::Path)?;
            if self
                .fetch
                .config
                .network
                .requires_proxy(self.fetch.config.proxy.is_some(), &uri)
            {
                return Err(self.failure(Reason::Proxy, None).into());
            }
        }
        if self.budget().expired() {
            return Err(self.failure(Reason::DeadlineExceeded, None).into());
        }
        Ok(())
    }
    pub(crate) fn send<T: Reply>(
        &self,
        ep: &Endpoint,
        mut prepared: Prepared,
        codec: Read<T>,
    ) -> Result<(T, crate::ReplyMetadata), RequestError> {
        self.spec.admit(&prepared.url, ep.publisher)?;
        prepared.identity_encoding = ep.identity_encoding;
        prepared.exact_count = ep.exact_count;
        if ep.exact_count {
            use sha2::Digest;
            let mut hash = sha2::Sha256::new();
            hash.update(b"Prefer:count=exact\0");
            hash.update(prepared.identity);
            prepared.identity = hash.finalize().into();
        }
        prepared.identity = self.licensed_identity(prepared.identity);
        prepared.publisher = ep.publisher;
        let mut codec = codec.octet_stream(ep.octet_stream);
        if let Some(limit) = ep.byte_limit {
            if limit == 0 || limit == u64::MAX {
                return Err(InputError::BodyLimit.into());
            }
            codec.limit = Some(codec.limit.map_or(limit, |old| old.min(limit)));
        }
        codec.json_rejections = ep.json_rejections && codec.name() == "application/json";
        codec.json_absence = ep.json_absence && codec.name() == "application/json";
        let (reply, fetched, content_range, pagination) =
            self.cached(ep, &prepared, &codec, || self.attempts(&prepared, &codec))?;
        let metadata = crate::ReplyMetadata {
            identity: prepared.identity,
            fetched,
            content_range,
            pagination,
        };
        self.report
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replies
            .push(metadata.clone());
        Ok((reply, metadata))
    }

    fn attempts<T: Reply>(
        &self,
        prepared: &Prepared,
        codec: &Read<T>,
    ) -> Result<Decoded<T>, RequestError> {
        for attempt in 0..crate::retry::ATTEMPTS {
            let permit = self
                .acquire_quota()
                .map_err(|delay| self.quota_failure(delay))?
                .ok_or_else(|| self.failure(Reason::DeadlineExceeded, None))?;
            let started = Instant::now();
            let exchange = self.exchange(prepared, codec);
            self.observe_exchange::<T>(prepared, &exchange, started.elapsed(), codec);
            let (mut error, retryable, delay) = match exchange {
                Ok(response) => {
                    let status = response.status;
                    if (200..300).contains(&status)
                        || codec.json_rejections && matches!(status, 400 | 422)
                        || codec.json_absence && status == 404
                    {
                        return self.decode(response, codec).map_err(Into::into);
                    }
                    let delay = response.retry_after;
                    (
                        self.failure(crate::retry::status_reason(status), Some(status)),
                        crate::retry::retry_status(status),
                        delay,
                    )
                }
                Err(Failure::Local(error)) => return Err(error),
                Err(Failure::Transport(error)) => {
                    let retryable = crate::retry::retry_error(&error);
                    let reason = crate::tls::reason(&error);
                    let mut failure = self.failure(reason, None);
                    failure.tls_cause = crate::tls::cause(&error);
                    (failure, retryable, None)
                }
            };
            drop(permit);
            if self.budget().expired() {
                return Err(self.failure(Reason::DeadlineExceeded, error.status).into());
            }
            error.retry_after = delay;
            if prepared.idempotent
                && retryable
                && delay.is_some_and(|wait| wait >= self.budget().remaining())
            {
                error.reason = Reason::RateLimited;
                return Err(error.into());
            }
            if !prepared.idempotent || !retryable || attempt + 1 == crate::retry::ATTEMPTS {
                return Err(error.into());
            }
            let wait = delay.unwrap_or_else(|| crate::retry::backoff(&prepared.identity, attempt));
            if !crate::retry::wait(wait, self.budget()) {
                return Err(self.failure(Reason::DeadlineExceeded, error.status).into());
            }
        }
        // Every iteration either returns or advances to the next attempt.
        Err(self.failure(Reason::Unavailable, None).into())
    }

    fn observe_exchange<T: Reply>(
        &self,
        prepared: &Prepared,
        exchange: &Result<Wire, Failure>,
        elapsed: Duration,
        codec: &Read<T>,
    ) {
        let (status, body, media) = match exchange {
            Ok(response) => (
                Some(response.status),
                response.body.as_slice(),
                response.content_type.as_str(),
            ),
            Err(Failure::Local(RequestError::Source(error))) => (error.status, &[][..], ""),
            Err(_) => (None, &[][..], ""),
        };
        if self.fetch.config.observations.is_some() {
            self.observed(
                &prepared.identity,
                status,
                body.len() as u64,
                crate::Shape::inspect(body, media),
                elapsed,
                (
                    std::any::type_name::<T>(),
                    false,
                    &prepared.url,
                    codec.provenance(prepared, body, status),
                    match exchange {
                        Err(Failure::Transport(error)) => crate::tls::cause(error),
                        Err(Failure::Local(RequestError::Source(error))) => error.tls_cause,
                        _ => None,
                    },
                ),
            );
        }
    }

    fn exchange<T: Reply>(&self, prepared: &Prepared, codec: &Read<T>) -> Result<Wire, Failure> {
        let local = |error| Failure::Local(RequestError::Recording(error));
        if let Some(exchange) = self
            .fetch
            .recordings
            .replay(
                self.spec.source,
                &prepared.identity,
                &prepared.url,
                prepared.recording(),
                codec.limit.unwrap_or(self.fetch.config.max_body),
            )
            .map_err(|error| {
                if codec.limit.is_some() && error == crate::RecordError::Limit {
                    Failure::Local(self.failure(codec.overflow_reason, None).into())
                } else {
                    local(error)
                }
            })?
        {
            return self.replayed_wire(prepared, exchange);
        }
        let started = Instant::now();
        let mut response = if prepared.publisher {
            self.publisher_call(&prepared.url, codec.name())?
        } else {
            self.wire_call(prepared, codec.name(), &[])
                .map_err(Failure::Transport)?
        };
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned();
        let content_range = response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let pagination = self.pagination_headers(prepared, &response, status)?;
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| crate::retry::retry_after(v, std::time::SystemTime::now()));
        let fetched = crate::cache::timestamp();
        let recording = matches!(self.fetch.mode, crate::Mode::Record { .. });
        let body = if prepared.method == Method::Head {
            Vec::new()
        } else {
            self.read_body(
                &mut response,
                status,
                recording
                    || self.keyed()
                    || !prepared.outgoing.is_empty()
                    || codec.json_rejections && matches!(status, 400 | 422)
                    || codec.json_absence && status == 404,
                codec.limit.unwrap_or(self.fetch.config.max_body),
                codec.overflow_reason,
            )?
        };
        let wire = Wire {
            fetched,
            status,
            content_type,
            content_range,
            pagination,
            body,
            retry_after,
        };
        self.contain_response(prepared, &wire, &wire.body, status)
            .map_err(Failure::Local)?;
        if recording {
            self.record_wire(prepared, codec, &wire, started)
                .map_err(local)?;
        }
        Ok(wire)
    }
    fn replayed_wire(&self, prepared: &Prepared, mut exchange: Exchange) -> Result<Wire, Failure> {
        if !self
            .budget()
            .advance(Duration::from_millis(exchange.delay_ms))
        {
            return Err(Failure::Local(
                self.failure(Reason::DeadlineExceeded, Some(exchange.status))
                    .into(),
            ));
        }
        // Sidecars become non-serializable Bytes on read. Scan their raw body
        // separately while deriving metadata from every remaining exchange field.
        let body = std::mem::replace(&mut exchange.body, Body::Text(String::new()));
        self.contain_response(prepared, &exchange, body.bytes(), exchange.status)
            .map_err(Failure::Local)?;
        Ok(Wire {
            fetched: exchange.fetched,
            status: exchange.status,
            content_type: exchange.content_type,
            content_range: exchange.content_range,
            pagination: exchange.pagination,
            body: body.bytes().to_vec(),
            retry_after: exchange.retry_after_ms.map(Duration::from_millis),
        })
    }
    fn record_wire<T: Reply>(
        &self,
        prepared: &Prepared,
        codec: &Read<T>,
        wire: &Wire,
        started: Instant,
    ) -> Result<(), crate::RecordError> {
        let exchange = Exchange {
            rpc: None,
            fetched: wire.fetched,
            status: wire.status,
            content_type: wire.content_type.clone(),
            content_range: wire.content_range.clone(),
            pagination: wire.pagination.clone(),
            body: if codec.binary() {
                Body::Bytes(wire.body.clone())
            } else {
                Body::capture(wire.body.clone())?
            },
            delay_ms: crate::record::millis(started.elapsed())?,
            retry_after_ms: wire.retry_after.map(crate::record::millis).transpose()?,
        };
        self.fetch.recordings.append(
            self.spec.source,
            &prepared.identity,
            &prepared.url,
            prepared.recording(),
            exchange,
        )
    }

    fn decode<T: Reply>(&self, response: Wire, codec: &Read<T>) -> Result<Decoded<T>, SourceError> {
        let status = response.status;
        let bad = || self.failure(Reason::BadReply, Some(status));
        if !codec.accepts(&response.content_type) {
            return Err(bad());
        }
        if self.budget().expired() {
            return Err(self.failure(Reason::DeadlineExceeded, Some(status)));
        }
        codec
            .validate(&response.body)
            .map_err(|reason| self.failure(reason, Some(status)))?;
        let body = codec.normalized_body(&response.content_type, &response.body);
        if body.len() as u64 > codec.limit.unwrap_or(self.fetch.config.max_body) {
            return Err(self.failure(codec.overflow_reason, Some(status)));
        }
        let reply = codec
            .decode_response(&body, response.content_range.as_deref(), status)
            .ok_or_else(bad)?;
        if self.budget().expired() {
            return Err(self.failure(Reason::DeadlineExceeded, Some(status)));
        }
        let body = match body {
            std::borrow::Cow::Borrowed(_) => response.body,
            std::borrow::Cow::Owned(body) => body,
        };
        Ok(Decoded {
            reply,
            status,
            body,
            fetched: response.fetched,
            content_range: response.content_range,
            pagination: response.pagination,
        })
    }
}
