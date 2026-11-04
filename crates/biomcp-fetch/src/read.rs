//! Response decoding and request-bound admission options.
use crate::{
    Reply,
    xml::{Bytes, Codec, Probe, XmlReply, validate},
};
use std::borrow::Cow;

type BodyCheck<'a> = dyn Fn(&[u8]) -> Result<(), biomcp_types::Reason> + 'a;
type ReplyCheck<'a, T> = dyn Fn(&T) -> Result<(), crate::BadReply> + 'a;
type RangeCheck<'a, T> = dyn Fn(&T, crate::ContentRange, u16) -> Result<(), crate::BadReply> + 'a;
type BodyNormalizer = for<'a> fn(&str, &'a [u8]) -> Cow<'a, [u8]>;
/// Options shared by live, replay and cached response reads.
pub struct Read<'a, T> {
    pub(crate) diagnostic_inputs: Option<(crate::Identities, String)>,
    validate: Option<&'a BodyCheck<'a>>,
    check: Option<&'a ReplyCheck<'a, T>>,
    cache_if: Option<&'a dyn Fn(&T) -> bool>,
    codec: Codec,
    xml_media: Option<fn(&str) -> bool>,
    normalize_body: Option<BodyNormalizer>,
    pub(crate) limit: Option<u64>,
    pub(crate) overflow_reason: biomcp_types::Reason,
    pub(crate) json_rejections: bool,
    pub(crate) json_absence: bool,
    octet_stream: Option<&'static str>,
    parse: fn(&[u8]) -> Option<T>,
    range_check: Option<&'a RangeCheck<'a, T>>,
}
impl<T> std::fmt::Debug for Read<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Read")
            .field("media", &self.codec.name())
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}
impl<'a, T: Reply> Read<'a, T> {
    /// Check request-bound identity before cache admission and every reuse.
    pub fn check(mut self, check: &'a ReplyCheck<'a, T>) -> Self {
        self.check = Some(check);
        self
    }
    /// Decode exactly one JSON string containing the typed reply.
    pub fn json_string() -> Self {
        let mut read = Self::json();
        read.parse = |body| {
            let inner: String = serde_json::from_slice(body).ok()?;
            serde_json::from_str(&inner).ok()
        };
        read
    }
    /// Check provider-given count, bounds and status with each response.
    pub fn counted(mut self, check: &'a RangeCheck<'a, T>) -> Self {
        self.range_check = Some(check);
        self
    }
    pub(crate) fn exact_count(&self) -> bool {
        self.range_check.is_some()
    }
    /// Set a bounded observation and its overflow reason.
    pub fn bounded(mut self, limit: u64, overflow: biomcp_types::Reason) -> Self {
        self.limit = Some(limit);
        self.overflow_reason = overflow;
        self
    }
    /// Validate the original bytes before decoding and cache admission.
    pub fn validate_body(mut self, check: &'a BodyCheck<'a>) -> Self {
        self.validate = Some(check);
        self
    }
    /// Attach explicit discovery inputs to request observations.
    pub fn inputs(mut self, ids: crate::Identities, parameter: &str) -> Self {
        self.diagnostic_inputs = Some((ids, parameter.into()));
        self
    }

    pub(crate) fn decode_response(
        &self,
        body: &[u8],
        range: Option<&str>,
        status: u16,
    ) -> Option<T> {
        let reply = self.decode(body)?;
        reply.check_status(status).ok()?;
        if let Some(check) = self.range_check {
            check(&reply, range.and_then(crate::ContentRange::parse)?, status).ok()?;
        }
        Some(reply)
    }

    /// Decode a checked native JSON reply.
    pub fn json() -> Self {
        Self {
            codec: Codec::Json,
            xml_media: None,
            normalize_body: None,
            validate: None,
            check: None,
            cache_if: None,
            range_check: None,
            diagnostic_inputs: None,
            octet_stream: None,
            limit: None,
            overflow_reason: biomcp_types::Reason::BadReply,
            json_rejections: false,
            json_absence: false,
            parse: |body| serde_json::from_slice(body).ok(),
        }
    }
    /// Decode native JSON served as text/plain with the same typed checks.
    pub fn json_text() -> Self {
        let mut read = Self::json();
        read.codec = Codec::JsonText;
        read
    }

    /// Decode checked XML with source media and encoding rules.
    pub fn xml() -> Self
    where
        T: XmlReply,
    {
        Self {
            codec: Codec::Xml,
            xml_media: Some(T::accepts_media),
            normalize_body: Some(T::normalize_body),
            validate: None,
            check: None,
            cache_if: None,
            range_check: None,
            diagnostic_inputs: None,
            octet_stream: None,
            limit: None,
            overflow_reason: biomcp_types::Reason::BadReply,
            json_rejections: false,
            json_absence: false,
            parse: |body| {
                let text = std::str::from_utf8(body).ok()?;
                let text = text.strip_prefix('\u{feff}').unwrap_or(text);
                validate(text)?;
                T::decode_xml(text).ok()
            },
        }
    }
    /// Admit only replies accepted by this predicate to cache.
    pub fn cache_if(mut self, predicate: &'a dyn Fn(&T) -> bool) -> Self {
        self.cache_if = Some(predicate);
        self
    }
    pub(crate) fn cacheable(&self, reply: &T) -> bool {
        self.cache_if.is_none_or(|predicate| predicate(reply))
    }
    pub(crate) fn validate(&self, bytes: &[u8]) -> Result<(), biomcp_types::Reason> {
        self.validate.map_or(Ok(()), |check| check(bytes))
    }
    pub(crate) fn binary(&self) -> bool {
        matches!(self.codec, Codec::Bytes(_))
    }
    pub(crate) fn octet_stream(mut self, accepted: Option<&'static str>) -> Self {
        self.octet_stream = accepted;
        self
    }
    pub(crate) fn name(&self) -> &'static str {
        self.codec.name()
    }
    pub(crate) fn accepts(&self, media: &str) -> bool {
        self.xml_media
            .map_or_else(|| self.codec.accepts(media), |accepts| accepts(media))
            || self.octet_stream.is_some_and(|accepted| {
                media
                    .split(';')
                    .next()
                    .is_some_and(|value| value.trim().eq_ignore_ascii_case(accepted))
            })
    }
    pub(crate) fn normalized_body<'b>(&self, media: &str, body: &'b [u8]) -> Cow<'b, [u8]> {
        self.normalize_body
            .map_or(Cow::Borrowed(body), |normalize| normalize(media, body))
    }
    pub(crate) fn decode(&self, body: &[u8]) -> Option<T> {
        let reply = (self.parse)(body)?;
        reply.check().ok()?;
        if let Some(check) = self.check {
            check(&reply).ok()?;
        }
        Some(reply)
    }
}

impl Read<'_, Probe> {
    pub(crate) fn probe() -> Self {
        Self {
            codec: Codec::Probe,
            xml_media: None,
            normalize_body: None,
            validate: None,
            check: None,
            cache_if: None,
            range_check: None,
            diagnostic_inputs: None,
            limit: None,
            overflow_reason: biomcp_types::Reason::BadReply,
            json_rejections: false,
            json_absence: false,
            octet_stream: None,
            parse: |_| Some(Probe),
        }
    }
}

impl<'a> Read<'a, Bytes> {
    /// Select the declared binary document media.
    pub fn document_media(mut self, media: crate::DocumentMedia) -> Self {
        self.codec = Codec::Bytes(media.media());
        self
    }

    /// Read opaque bytes under an explicit bound.
    pub fn bytes() -> Self {
        Self {
            codec: Codec::Bytes("application/octet-stream"),
            xml_media: None,
            normalize_body: None,
            validate: None,
            check: None,
            cache_if: None,
            range_check: None,
            diagnostic_inputs: None,
            octet_stream: None,
            limit: None,
            overflow_reason: biomcp_types::Reason::BadReply,
            json_rejections: false,
            json_absence: false,
            parse: |body| Some(Bytes(body.to_vec())),
        }
    }
}
