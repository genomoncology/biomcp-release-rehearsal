use super::{
    Bundle, BundleSync, MAX_BUNDLE_BYTES, Row, parse,
    store::{self, Meta, Part},
};
use crate::{
    CacheError, Deadline, Leg, Mode, RequestError,
    declaration::Method,
    prepared::Prepared,
    record::{Body, Exchange},
};
use biomcp_types::Reason;
use std::{
    fs,
    io::{Read, Seek, Write},
    time::{Duration, Instant},
};
struct Transfer<'a> {
    holder: &'a str,
    file: &'a mut tempfile::NamedTempFile,
    part: &'a mut Part,
    auto: bool,
    downloaded_bytes: u64,
    complete: bool,
    started: Instant,
}
impl Leg<'_> {
    pub(super) fn sync_bundle<R: Row>(
        &self,
        b: &Bundle<R>,
        auto: bool,
    ) -> Result<BundleSync, RequestError> {
        self.bundle_source(b)?;
        if matches!(self.fetch.mode, Mode::Offline) {
            return Err(self.failure(Reason::Offline, None).into());
        }
        let cache = self.bundle_cache()?;
        if matches!(self.fetch.mode, Mode::Replay { .. }) {
            return self.replay_bundle(b);
        }
        let old = if matches!(self.fetch.mode, Mode::Record { .. }) {
            None
        } else {
            cache.bundle_info(b.name, self.budget())?
        };
        if auto
            && old
                .as_ref()
                .is_some_and(|m| m.bytes > self.fetch.config.auto_sync_max_bytes)
        {
            return Err(self.failure(Reason::Syncing, None).into());
        }
        fs::create_dir_all(&cache.1).map_err(|_| CacheError::Filesystem)?;
        let mut file = store::create_part(&cache.1)?;
        let holder = file
            .path()
            .file_name()
            .ok_or(CacheError::Internal)?
            .to_string_lossy()
            .into_owned();
        let path = file.path().to_string_lossy().into_owned();
        let url = format!("{}{}", self.spec.base.trim_end_matches('/'), b.path);
        let lease = crate::cache::now().saturating_add(
            i64::try_from(self.fetch.config.sync_budget.as_secs())
                .unwrap_or(i64::MAX)
                .saturating_add(1),
        );
        let prior = match cache.claim(b.name, &url, &holder, &path, lease, self.budget()) {
            Err(CacheError::Busy) => return Err(self.failure(Reason::Syncing, None).into()),
            result => result?,
        };
        let mut part = Part {
            path,
            bytes: 0,
            etag: None,
            total: None,
        };
        let record = matches!(self.fetch.mode, Mode::Record { .. });
        resume(
            prior,
            &mut part,
            &mut file,
            !record && matches!(b.format, super::RowFormat::Tsv),
        )?;
        let mut transfer = Transfer {
            holder: &holder,
            file: &mut file,
            part: &mut part,
            auto,
            downloaded_bytes: 0,
            complete: false,
            started: Instant::now(),
        };
        let result = self.download(
            b,
            &url,
            &mut transfer,
            if record || !matches!(b.format, super::RowFormat::Tsv) {
                None
            } else {
                old.as_ref()
            },
        );
        if result.is_err() {
            // Preserve this holder's file only if its fenced metadata update succeeds.
            let cleanup = Deadline::start(Duration::from_secs(1), None);
            if transfer.complete {
                cache.discard(b.name, &holder, cleanup)?;
            } else if cache.retain(b.name, &holder, &part, cleanup)? {
                file.keep().map_err(|_| CacheError::Filesystem)?;
            }
        }
        result
    }
    fn replay_bundle<R: Row>(&self, b: &Bundle<R>) -> Result<BundleSync, RequestError> {
        let cache = self.bundle_cache()?;
        // The Fetch-local import set is independent of warm case storage.
        let mut imported = self
            .fetch
            .bundle_imports
            .lock()
            .map_err(|_| CacheError::Internal)?;
        if imported.contains(b.name) {
            return cache
                .bundle_info(b.name, self.budget())?
                .as_ref()
                .map(|m| BundleSync::Published(store::info(b, m)))
                .ok_or(CacheError::Internal.into());
        }
        let url = format!("{}{}", self.spec.base.trim_end_matches('/'), b.path);
        let key = crate::retry::identity(&url);
        let exchange = super::replay::exchange(self, b, &key, &url)?;
        if exchange.body.bytes().len() as u64 > R::MAX_BYTES {
            return Err(self.failure(Reason::BadReply, Some(200)).into());
        }
        let rows = parse::rows::<R>(exchange.body.bytes(), b.format)
            .map_err(|_| self.failure(Reason::BadReply, Some(200)))?;
        let mut meta = meta(
            b,
            exchange.body.bytes().len() as u64,
            rows.len() as u64,
            None,
            None,
        );
        meta.rejections = rows.rejections.clone();
        provenance(&mut meta, exchange.body.bytes());
        let result = self.publish_checked(b, None, &meta, &rows, 0)?;
        if matches!(result, BundleSync::Published(_)) {
            imported.insert(b.name);
        }
        Ok(result)
    }
    fn download<R: Row>(
        &self,
        b: &Bundle<R>,
        url: &str,
        t: &mut Transfer<'_>,
        old: Option<&Meta>,
    ) -> Result<BundleSync, RequestError> {
        let prepared = Prepared::new(url.into(), Vec::new(), Method::Get, true);
        let mut headers = conditional(t.part, old)?;
        for attempt in 0..crate::retry::ATTEMPTS {
            let permit = self
                .acquire_quota()
                .map_err(|delay| self.quota_failure(delay))?
                .ok_or_else(|| self.failure(Reason::Syncing, None))?;
            t.started = Instant::now();
            let response = self.wire_call(&prepared, b.format.media(), &headers);
            match response {
                Ok(mut response) => {
                    let status = response.status().as_u16();
                    self.observe_bundle_status(&prepared, status, b.format, t.started.elapsed());
                    if matches!(status, 412 | 416) && t.part.bytes > 0 {
                        reset(t.file, t.part)?;
                        headers.clear();
                        continue;
                    }
                    if status == 304
                        && let Some(old) = old
                    {
                        return self.revalidated(b, t.holder, old);
                    }
                    if status == 200 || status == 206 {
                        return self.consume(b, t, &mut response);
                    }
                    let wait = header(&response, "retry-after")
                        .and_then(|v| crate::retry::retry_after(&v, std::time::SystemTime::now()));
                    drop(permit);
                    if !crate::retry::retry_status(status) || attempt + 1 == crate::retry::ATTEMPTS
                    {
                        return Err(self
                            .failure(crate::retry::status_reason(status), Some(status))
                            .into());
                    }
                    if !crate::retry::wait(
                        wait.unwrap_or_else(|| crate::retry::backoff(&prepared.identity, attempt)),
                        self.budget(),
                    ) {
                        return Err(self.failure(Reason::Syncing, Some(status)).into());
                    }
                }
                Err(error) => {
                    self.observe(
                        (&prepared.identity, &prepared.url),
                        None,
                        &[],
                        b.format.media(),
                        t.started.elapsed(),
                    );
                    drop(permit);
                    if !crate::retry::retry_error(&error) || attempt + 1 == crate::retry::ATTEMPTS {
                        return Err(self
                            .failure(
                                if self.budget().expired() {
                                    Reason::Syncing
                                } else {
                                    crate::tls::reason(&error)
                                },
                                None,
                            )
                            .into());
                    }
                    if !crate::retry::wait(
                        crate::retry::backoff(&prepared.identity, attempt),
                        self.budget(),
                    ) {
                        return Err(self.failure(Reason::Syncing, None).into());
                    }
                }
            }
        }
        Err(self.failure(Reason::Syncing, None).into())
    }
    fn observe_bundle_status(
        &self,
        prepared: &Prepared,
        status: u16,
        format: super::RowFormat,
        elapsed: Duration,
    ) {
        if status != 200 && status != 206 {
            self.observe(
                (&prepared.identity, &prepared.url),
                Some(status),
                &[],
                format.media(),
                elapsed,
            );
        }
    }
    fn revalidated<R: Row>(
        &self,
        b: &Bundle<R>,
        holder: &str,
        old: &Meta,
    ) -> Result<BundleSync, RequestError> {
        let mut m = old.clone();
        m.fresh_until = fresh_until(b);
        if !self
            .bundle_cache()?
            .publish(b, Some(holder), &m, None, self.budget())?
        {
            return Err(self.failure(Reason::Syncing, None).into());
        }
        Ok(BundleSync::Published(store::info(b, &m)))
    }
    fn consume<R: Row>(
        &self,
        b: &Bundle<R>,
        t: &mut Transfer<'_>,
        response: &mut ureq::http::Response<ureq::Body>,
    ) -> Result<BundleSync, RequestError> {
        let status = response.status().as_u16();
        self.validate_range(t, response)?;
        if !matches!(b.format, super::RowFormat::Tsv) {
            t.part.etag = None;
        }
        self.stream(t, response, R::MAX_BYTES)?;
        t.complete = true;
        t.file.flush().map_err(|_| CacheError::Filesystem)?;
        t.file.rewind().map_err(|_| CacheError::Filesystem)?;
        let mut body = Vec::new();
        t.file
            .read_to_end(&mut body)
            .map_err(|_| CacheError::Filesystem)?;
        let url = format!("{}{}", self.spec.base.trim_end_matches('/'), b.path);
        self.observe_bundle(
            (&crate::retry::identity(&url), &url),
            Some(status),
            &body,
            b.format,
            t.started.elapsed(),
        );
        let rows = parse::rows::<R>(&body, b.format)
            .map_err(|_| self.failure(Reason::BadReply, Some(status)))?;
        if self.budget().expired() {
            return Err(self.failure(Reason::Syncing, None).into());
        }
        let mut meta = meta(
            b,
            t.part.bytes,
            rows.len() as u64,
            header(response, "etag"),
            header(response, "last-modified"),
        );
        meta.rejections = rows.rejections.clone();
        provenance(&mut meta, &body);
        if matches!(self.fetch.mode, Mode::Record { .. }) {
            let exchange = Exchange {
                rpc: None,
                fetched: crate::cache::timestamp(),
                status: 200,
                content_type: b.format.media().into(),
                content_range: None,
                pagination: None,
                delay_ms: crate::record::millis(t.started.elapsed())
                    .map_err(RequestError::Recording)?,
                retry_after_ms: None,
                body: Body::capture_bundle(body).map_err(RequestError::Recording)?,
            };
            let url = format!("{}{}", self.spec.base.trim_end_matches('/'), b.path);
            self.fetch
                .recordings
                .append_bundle(
                    self.spec.source,
                    &crate::retry::identity(&url),
                    &url,
                    exchange,
                )
                .map_err(RequestError::Recording)?;
        }
        self.publish_checked(b, Some(t.holder), &meta, &rows, t.downloaded_bytes)
    }
    fn publish_checked<R: Row>(
        &self,
        b: &Bundle<R>,
        holder: Option<&str>,
        meta: &Meta,
        rows: &parse::ParsedRows,
        downloaded_bytes: u64,
    ) -> Result<BundleSync, RequestError> {
        let cache = self.bundle_cache()?;
        if R::REQUIRE_ACCEPTED_ROWS && rows.is_empty() && !rows.rejections.is_empty() {
            if let Some(holder) = holder {
                cache.discard(b.name, holder, self.budget())?;
            }
            let retained = cache
                .bundle_info(b.name, self.budget())?
                .as_ref()
                .map(|meta| store::info(b, meta));
            return Ok(BundleSync::Refused {
                native_rows: rows.rejections.len() as u64,
                rejections: rows.rejections.clone(),
                bytes: meta.bytes,
                downloaded_bytes,
                retained,
            });
        }
        if !cache.publish(b, holder, meta, Some(rows), self.budget())? {
            return Err(self.failure(Reason::Syncing, None).into());
        }
        let mut info = store::info(b, meta);
        info.downloaded_bytes = downloaded_bytes;
        Ok(BundleSync::Published(info))
    }
    fn validate_range(
        &self,
        t: &mut Transfer<'_>,
        response: &ureq::http::Response<ureq::Body>,
    ) -> Result<(), RequestError> {
        let status = response.status().as_u16();
        let etag = header(response, "etag");
        let length = header(response, "content-length").and_then(|v| v.parse::<u64>().ok());
        if status == 206 {
            let Some((start, end, total)) =
                header(response, "content-range").and_then(|v| range(&v))
            else {
                reset(t.file, t.part)?;
                return Err(self.failure(Reason::BadReply, Some(status)).into());
            };
            if t.part.bytes == 0
                || start != t.part.bytes
                || !strong(etag.as_deref())
                || etag != t.part.etag
                || t.part.total.is_some_and(|n| n != total)
                || length.is_some_and(|n| n != end - start + 1)
            {
                reset(t.file, t.part)?;
                return Err(self.failure(Reason::BadReply, Some(status)).into());
            }
            if total > MAX_BUNDLE_BYTES {
                reset(t.file, t.part)?;
                return Err(self.failure(Reason::Syncing, Some(status)).into());
            }
            t.part.total = Some(total);
        } else {
            reset(t.file, t.part)?;
            if length.is_some_and(|n| n > MAX_BUNDLE_BYTES) {
                return Err(self.failure(Reason::Syncing, Some(status)).into());
            }
            t.part.total = length;
            t.part.etag = etag;
        }
        Ok(())
    }
    fn stream(
        &self,
        t: &mut Transfer<'_>,
        response: &mut ureq::http::Response<ureq::Body>,
        source_limit: u64,
    ) -> Result<(), RequestError> {
        let status = response.status().as_u16();
        let limit = if t.auto {
            self.fetch.config.auto_sync_max_bytes.min(MAX_BUNDLE_BYTES)
        } else {
            MAX_BUNDLE_BYTES
        }
        .min(source_limit);
        if t.part.total.is_some_and(|n| n > limit) {
            return Err(self.failure(Reason::Syncing, Some(status)).into());
        }
        let mut reader = response.body_mut().as_reader();
        let mut buffer = [0u8; 16 * 1024];
        loop {
            if self.budget().expired() {
                return Err(self.failure(Reason::Syncing, Some(status)).into());
            }
            let n = reader.read(&mut buffer).map_err(|_| {
                self.failure(
                    if self.budget().expired() {
                        Reason::Syncing
                    } else {
                        Reason::BadReply
                    },
                    Some(status),
                )
            })?;
            if n == 0 {
                break;
            }
            let Some(bytes) = buffer.get(..n) else {
                return Err(self.failure(Reason::BadReply, Some(status)).into());
            };
            if t.part.bytes.saturating_add(n as u64) > limit {
                return Err(self.failure(Reason::Syncing, Some(status)).into());
            }
            t.file
                .write_all(bytes)
                .map_err(|_| CacheError::Filesystem)?;
            t.part.bytes += n as u64;
            t.downloaded_bytes += n as u64;
        }
        if t.part.total.is_some_and(|n| n != t.part.bytes) {
            return Err(self.failure(Reason::BadReply, Some(status)).into());
        }
        Ok(())
    }
}
fn reset(file: &mut tempfile::NamedTempFile, part: &mut Part) -> Result<(), CacheError> {
    use std::io::{Seek, SeekFrom};
    file.as_file_mut()
        .set_len(0)
        .map_err(|_| CacheError::Filesystem)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_| CacheError::Filesystem)?;
    part.bytes = 0;
    part.etag = None;
    part.total = None;
    Ok(())
}
fn strong(etag: Option<&str>) -> bool {
    etag.is_some_and(|e| {
        e.starts_with('"')
            && e.ends_with('"')
            && e.len() > 2
            && !e.contains('\r')
            && !e.contains('\n')
    })
}
fn header(response: &ureq::http::Response<ureq::Body>, name: &str) -> Option<String> {
    response
        .headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}
fn range(value: &str) -> Option<(u64, u64, u64)> {
    let (span, total) = value.strip_prefix("bytes ")?.split_once('/')?;
    let (start, end) = span.split_once('-')?;
    let (start, end, total) = (start.parse().ok()?, end.parse().ok()?, total.parse().ok()?);
    (start <= end && end < total).then_some((start, end, total))
}
fn fresh_until<R: Row>(b: &Bundle<R>) -> i64 {
    crate::cache::now()
        .saturating_add(i64::try_from(b.fresh.seconds().unwrap_or(0)).unwrap_or(i64::MAX))
}
fn meta<R: Row>(
    b: &Bundle<R>,
    bytes: u64,
    rows: u64,
    etag: Option<String>,
    modified: Option<String>,
) -> Meta {
    Meta {
        rejections: Vec::new(),
        bytes,
        rows,
        fresh_until: fresh_until(b),
        etag,
        modified,
        content_revision: None,
        retrieved_at: None,
    }
}

fn conditional(part: &Part, old: Option<&Meta>) -> Result<Vec<(&'static str, String)>, CacheError> {
    let mut headers = Vec::new();
    if part.bytes > 0 {
        headers.push(("Range", format!("bytes={}-", part.bytes)));
        headers.push(("If-Match", part.etag.clone().ok_or(CacheError::Internal)?));
    } else if let Some(old) = old {
        if let Some(etag) = &old.etag {
            headers.push(("If-None-Match", etag.clone()));
        } else if let Some(date) = &old.modified {
            headers.push(("If-Modified-Since", date.clone()));
        }
    }
    Ok(headers)
}

fn resume(
    prior: Option<Part>,
    part: &mut Part,
    file: &mut tempfile::NamedTempFile,
    allowed: bool,
) -> Result<(), CacheError> {
    if let Some(prior) = prior {
        if allowed
            && strong(prior.etag.as_deref())
            && prior.bytes > 0
            && let Ok(input) = fs::File::open(&prior.path)
        {
            let copied = std::io::copy(&mut input.take(prior.bytes), file)
                .map_err(|_| CacheError::Filesystem)?;
            if copied == prior.bytes {
                part.bytes = copied;
                part.etag = prior.etag;
                part.total = prior.total;
            } else {
                reset(file, part)?;
            }
        }
        // claim transfers the validated predecessor's file to this holder.
        if let Err(error) = fs::remove_file(prior.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            return Err(CacheError::Filesystem);
        }
    }
    Ok(())
}

fn provenance(meta: &mut Meta, body: &[u8]) {
    let hex = super::bundle_sha256(body);
    meta.content_revision = Some(format!("sha256:{hex}"));
    meta.retrieved_at = crate::cache::timestamp();
}
