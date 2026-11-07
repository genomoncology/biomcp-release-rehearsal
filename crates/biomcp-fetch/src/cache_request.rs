//! Checked cache reuse around the single live request path.
use crate::read::Read;
use crate::{
    Endpoint, Leg, Mode, Reply, RequestError,
    cache::{Cache, Entry, now},
};
use biomcp_types::Reason;

type CachedReply<T> = (
    T,
    Option<i64>,
    Option<crate::ContentRange>,
    Option<crate::Pagination>,
);

const NEGATIVE_SECONDS: u64 = 600;
enum Checked<T> {
    Positive(
        T,
        bool,
        Option<i64>,
        Option<crate::ContentRange>,
        Option<crate::Pagination>,
    ),
    Absent,
}

impl Leg<'_> {
    fn cache_result(&self, result: Result<(), crate::CacheError>) {
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

    pub(crate) fn cached<T: Reply>(
        &self,
        endpoint: &Endpoint,
        prepared: &crate::prepared::Prepared,
        codec: &Read<T>,
        live: impl FnOnce() -> Result<crate::request::Decoded<T>, RequestError>,
    ) -> Result<CachedReply<T>, RequestError> {
        let (url, key) = (prepared.url.as_str(), &prepared.identity);
        if self.budget().expired() {
            return Err(self.failure(Reason::DeadlineExceeded, None).into());
        }
        if self.fetch.mode == Mode::Offline {
            return self.offline(endpoint, prepared, codec);
        }
        let Some((cache, seconds)) = self.fetch.cache.as_ref().zip(endpoint.fresh.seconds()) else {
            return live().map(|decoded| {
                let crate::request::Decoded {
                    reply,
                    fetched,
                    content_range: range,
                    pagination,
                    ..
                } = decoded;
                (
                    reply,
                    fetched,
                    range.as_deref().and_then(crate::ContentRange::parse),
                    pagination,
                )
            });
        };
        let mut stale = None;
        if matches!(self.fetch.mode, Mode::Live { .. } | Mode::Replay { .. }) && !self.fresh {
            match self.checked_cached(cache, prepared, codec)? {
                Some(Checked::Positive(reply, true, fetched, range, pagination)) => {
                    return Ok((reply, fetched, range, pagination));
                }
                Some(Checked::Positive(reply, false, fetched, range, pagination)) => {
                    stale = Some((reply, fetched, range, pagination))
                }
                Some(Checked::Absent) => {
                    return Err(self.failure(Reason::NotFound, Some(404)).into());
                }
                None => {}
            }
        }
        match live() {
            Ok(decoded) => self.admit_live(cache, prepared, codec, seconds, decoded),
            Err(RequestError::Source(error)) if error.reason == Reason::NotFound => {
                // A current absence replaces a formerly positive entry; no stale
                // result can override it and the provider body is never stored.
                self.cache_result(cache.write(
                    key,
                    self.spec.source,
                    url,
                    &Entry::new(404, Vec::new(), NEGATIVE_SECONDS, codec.name()),
                    self.budget(),
                ));
                Err(error.into())
            }
            Err(RequestError::Source(error)) => {
                if let Some(reply) = stale
                    && !(matches!(self.spec.key, crate::KeyUse::Required(_))
                        && matches!(error.status, Some(401 | 403)))
                {
                    let mut report = self
                        .report
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    report.stale = true;
                    report.failures.push(error);
                    Ok(reply)
                } else {
                    Err(error.into())
                }
            }
            Err(error) => Err(error),
        }
    }

    fn admit_live<T: Reply>(
        &self,
        cache: &Cache,
        prepared: &crate::prepared::Prepared,
        codec: &Read<T>,
        seconds: u64,
        decoded: crate::request::Decoded<T>,
    ) -> Result<CachedReply<T>, RequestError> {
        let crate::request::Decoded {
            reply,
            status,
            body,
            fetched,
            content_range: range,
            pagination,
        } = decoded;
        let mut entry = Entry::new(status, body, seconds, codec.name());
        if codec.json_absence && status == 404 {
            self.cache_result(cache.remove(&prepared.identity, self.budget()));
        }
        // A recording without acquisition time cannot establish cache freshness.
        let Some(acquired) = fetched else {
            return Ok((
                reply,
                None,
                range.as_deref().and_then(crate::ContentRange::parse),
                pagination,
            ));
        };
        entry.fetched_at = acquired;
        entry.content_range = range;
        entry.pagination = pagination;
        if (200..300).contains(&status) && codec.cacheable(&reply) && !self.license_unchecked() {
            self.cache_result(cache.write(
                &prepared.identity,
                self.spec.source,
                &prepared.url,
                &entry,
                self.budget(),
            ));
        }
        Ok((
            reply,
            fetched,
            entry
                .content_range
                .as_deref()
                .and_then(crate::ContentRange::parse),
            entry.pagination,
        ))
    }

    fn offline<T: Reply>(
        &self,
        endpoint: &Endpoint,
        prepared: &crate::prepared::Prepared,
        codec: &Read<T>,
    ) -> Result<CachedReply<T>, RequestError> {
        if let Some(cache) = self
            .fetch
            .cache
            .as_ref()
            .filter(|_| endpoint.fresh.seconds().is_some())
        {
            match self.checked_cached(cache, prepared, codec)? {
                Some(Checked::Positive(reply, fresh, fetched, range, pagination)) => {
                    if !fresh {
                        self.report
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .stale = true;
                    }
                    return Ok((reply, fetched, range, pagination));
                }
                Some(Checked::Absent) => {
                    return Err(self.failure(Reason::NotFound, Some(404)).into());
                }
                None => {}
            }
        }
        Err(self.failure(Reason::Offline, None).into())
    }

    fn checked_cached<T: Reply>(
        &self,
        cache: &Cache,
        prepared: &crate::prepared::Prepared,
        codec: &Read<T>,
    ) -> Result<Option<Checked<T>>, RequestError> {
        let key = &prepared.identity;
        let entry = match cache.read(
            key,
            codec.limit.unwrap_or(self.fetch.config.max_body),
            self.budget(),
            !matches!(self.fetch.mode, Mode::Offline | Mode::Replay { .. }),
        ) {
            Ok(entry) => entry,
            Err(crate::CacheError::Busy) => {
                self.cache_result(Err(crate::CacheError::Busy));
                None
            }
            Err(error) => return Err(error.into()),
        };
        let Some(mut entry) = entry else {
            return Ok(None);
        };
        self.contain_response(prepared, &entry, &entry.body, entry.status)?;
        self.filter_cached(cache, prepared, codec, &mut entry)?;
        let fresh = entry.fresh_until > now();
        if entry.status == 404 {
            if fresh {
                self.observed(
                    key,
                    Some(404),
                    0,
                    crate::Shape::default(),
                    std::time::Duration::ZERO,
                    (
                        std::any::type_name::<T>(),
                        true,
                        &prepared.url,
                        codec.provenance(prepared, &entry.body, Some(entry.status)),
                        None,
                    ),
                );
            }
            return Ok(fresh.then_some(Checked::Absent));
        }
        let reply = (entry.content_type.as_deref() == Some(codec.name())
            && codec.validate(&entry.body).is_ok())
        .then(|| codec.decode_response(&entry.body, entry.content_range.as_deref(), entry.status))
        .flatten()
        .filter(|reply| codec.cacheable(reply));
        let Some(reply) = reply else {
            // A type or limit change invalidates this entry. It is a miss and
            // cannot become a stale fallback after the live request fails.
            self.cache_result(cache.remove(key, self.budget()));
            return Ok(None);
        };
        if self.budget().expired() {
            return Err(self
                .failure(Reason::DeadlineExceeded, Some(entry.status))
                .into());
        }
        if self.fetch.config.observations.is_some() && (fresh || self.fetch.mode == Mode::Offline) {
            self.observed(
                key,
                Some(entry.status),
                0,
                crate::Shape::inspect(&entry.body, codec.name()),
                std::time::Duration::ZERO,
                (
                    std::any::type_name::<T>(),
                    true,
                    &prepared.url,
                    codec.provenance(prepared, &entry.body, Some(entry.status)),
                    None,
                ),
            );
        }
        Ok(Some(Checked::Positive(
            reply,
            fresh,
            Some(entry.fetched_at),
            entry
                .content_range
                .as_deref()
                .and_then(crate::ContentRange::parse),
            entry.pagination,
        )))
    }
    /// Sanitize a checked positive entry before either reuse or a persistent rewrite.
    fn filter_cached<T: Reply>(
        &self,
        cache: &Cache,
        prepared: &crate::prepared::Prepared,
        codec: &Read<T>,
        entry: &mut Entry,
    ) -> Result<(), RequestError> {
        if (200..300).contains(&entry.status) {
            let filtered = codec
                .filtered_body(&entry.body)
                .map_err(|_| self.failure(Reason::BadReply, Some(entry.status)))?;
            if filtered.len() as u64 > codec.limit.unwrap_or(self.fetch.config.max_body) {
                return Err(self
                    .failure(codec.overflow_reason, Some(entry.status))
                    .into());
            }
            if filtered.as_ref() != entry.body.as_slice() {
                entry.body = filtered.into_owned();
                // Text filtered on an unfinished lookup serves this read only.
                if !self.license_unchecked() {
                    cache.write(
                        &prepared.identity,
                        self.spec.source,
                        &prepared.url,
                        entry,
                        self.budget(),
                    )?;
                }
            }
        }
        Ok(())
    }
}
