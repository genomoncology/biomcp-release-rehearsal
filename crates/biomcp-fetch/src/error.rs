use biomcp_types::{Reason, Source};
use std::fmt;

/// A rejected decoded reply; carries no provider text.
#[derive(Clone, Copy, Debug)]
pub struct BadReply;
impl fmt::Display for BadReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fetch.bad_reply")
    }
}
impl std::error::Error for BadReply {}

/// A typed source failure containing no request or provider text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceError {
    /// Source whose request failed.
    pub source: Source,
    /// Stable failure reason.
    pub reason: Reason,
    /// HTTP status when a response was received.
    pub status: Option<u16>,
    /// Safe certificate or handshake cause, present only for TLS failures.
    pub tls_cause: Option<crate::TlsCause>,
    /// Provider retry delay or remaining local quota spacing, when present.
    pub retry_after: Option<std::time::Duration>,
}

impl SourceError {
    /// Describe a TLS failure using the shared fixed cause vocabulary.
    pub fn tls_notice(self) -> Option<biomcp_types::SourceNotice> {
        self.tls_cause
            .filter(|_| self.reason == Reason::Tls)
            .map(|cause| biomcp_types::SourceNotice::RequestGuidance {
                message: format!("TLS verification failed: {cause}").into(),
                retry_after_seconds: None,
            })
    }
    /// Refuse a source disabled by caller policy before any source access.
    pub fn disabled(source: Source) -> Self {
        Self {
            source,
            reason: Reason::Disabled,
            status: None,
            tls_cause: None,
            retry_after: None,
        }
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.source, self.reason)?;
        if let Some(cause) = self.tls_cause {
            write!(f, ": {cause}")?;
        }
        Ok(())
    }
}

impl std::error::Error for SourceError {}

/// Invalid explicit transport configuration, without retaining supplied text.
#[derive(Clone, Copy, Debug)]
pub enum FetchError {
    /// Streaming allocation limits must describe finite supported resources.
    StreamingLimits,
    /// The configured proxy could not be parsed.
    Proxy,
    /// The HTTP proxy environment setting is invalid.
    HttpProxy,
    /// The HTTPS proxy environment setting is invalid.
    HttpsProxy,
    /// A bypass rule contains credentials or a malformed port.
    ProxyBypass,
    /// The configured additive certificates could not be checked.
    Certificates,
    /// The body limit must allow one byte for overflow detection.
    BodyLimit,
    /// The explicit cache could not be opened safely.
    Cache(CacheError),
    /// Local recording ownership or content failure.
    Recording(RecordError),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StreamingLimits => "fetch.streaming_limits",
            Self::Proxy => "fetch.proxy",
            Self::HttpProxy => "fetch.http_proxy",
            Self::HttpsProxy => "fetch.https_proxy",
            Self::ProxyBypass => "fetch.proxy_bypass",
            Self::Certificates => "fetch.certificates",
            Self::BodyLimit => "fetch.body_limit",
            Self::Cache(error) => return error.fmt(f),
            Self::Recording(error) => return error.fmt(f),
        })
    }
}

impl std::error::Error for FetchError {}

/// Local cache failures carry no SQL, file paths or provider text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheError {
    /// A SQLite operation failed.
    Database,
    /// Cache JSON could not be encoded or decoded.
    Encoding,
    /// A cache filesystem operation failed.
    Filesystem,
    /// Cache state or synchronization was inconsistent.
    Internal,
    /// Cache content exceeded a supported bound or failed validation.
    Limit,
    /// A stored snapshot could not be encoded or decoded.
    Snapshot,
    /// The file has no valid transport ownership or a different schema version.
    Version,
    /// The connection or database remained busy until the request deadline.
    Busy,
}
impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Database => "fetch.cache_database",
            Self::Encoding => "fetch.cache_encoding",
            Self::Filesystem => "fetch.cache_filesystem",
            Self::Internal => "fetch.cache_internal",
            Self::Limit => "fetch.cache_limit",
            Self::Snapshot => "fetch.cache_snapshot",
            Self::Version => "fetch.cache_version",
            Self::Busy => "fetch.cache_busy",
        })
    }
}
impl std::error::Error for CacheError {}
impl From<CacheError> for biomcp_types::CacheProblem {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::Busy => Self::Busy,
            CacheError::Database => Self::Database,
            CacheError::Encoding => Self::Encoding,
            CacheError::Filesystem => Self::Filesystem,
            CacheError::Internal => Self::Internal,
            CacheError::Limit => Self::Limit,
            CacheError::Snapshot => Self::Snapshot,
            CacheError::Version => Self::Version,
        }
    }
}

/// A provider failure or a distinct local cache failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestError {
    /// Local request preparation failed before storage or HTTP.
    Input(InputError),
    /// The provider request failed.
    Source(SourceError),
    /// Local cache access or validation failed.
    Cache(CacheError),
    /// Local recording ownership or content failure.
    Recording(RecordError),
}
impl From<SourceError> for RequestError {
    fn from(error: SourceError) -> Self {
        Self::Source(error)
    }
}
impl From<CacheError> for RequestError {
    fn from(error: CacheError) -> Self {
        Self::Cache(error)
    }
}
impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(error) => error.fmt(f),
            Self::Source(error) => error.fmt(f),
            Self::Cache(error) => error.fmt(f),
            Self::Recording(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for RequestError {}

/// Local recording failures never retain paths, URLs or provider text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
    /// The capture directory is nonempty or already owned.
    Conflict,
    /// Local recording I/O failed.
    Io(std::io::ErrorKind),
    /// No file matches this source and request identity.
    Missing,
    /// Every exchange for this request was consumed.
    Exhausted,
    /// Recording metadata or exchange content is invalid.
    Malformed,
    /// Recording size or sequence bounds were exceeded.
    Limit,
    /// Binary capture needs a future sidecar representation.
    NonUtf8,
}
impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Conflict => "fetch.record_conflict",
            Self::Io(_) => "fetch.record_io",
            Self::Missing => "fetch.record_missing",
            Self::Exhausted => "fetch.record_exhausted",
            Self::Malformed => "fetch.record_malformed",
            Self::Limit => "fetch.record_limit",
            Self::NonUtf8 => "fetch.record_non_utf8",
        })
    }
}
impl std::error::Error for RecordError {}
impl From<RecordError> for RequestError {
    fn from(error: RecordError) -> Self {
        Self::Recording(error)
    }
}

/// Input failures retain no caller text or serializer diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputError {
    /// A configured credential is not valid UTF-8.
    CredentialEncoding,
    /// The request origin is absent from its source declaration.
    Host,
    /// A runtime endpoint path is not canonical or attempts to supply URL parts.
    Path,
    /// The call does not match the endpoint's declared body encoding.
    Method,
    /// The caller's JSON serializer failed.
    Serialization,
    /// The encoded request exceeds the configured body limit.
    BodyLimit,
}
impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CredentialEncoding => "fetch.input_credential_encoding",
            Self::Host => "fetch.input_host",
            Self::Path => "fetch.input_path",
            Self::Method => "fetch.input_method",
            Self::Serialization => "fetch.input_serialization",
            Self::BodyLimit => "fetch.input_body_limit",
        })
    }
}
impl std::error::Error for InputError {}
impl From<InputError> for RequestError {
    fn from(error: InputError) -> Self {
        Self::Input(error)
    }
}

impl From<std::io::Error> for RecordError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.kind())
    }
}

/// Construct a rejected source reply while retaining its known HTTP status.
pub fn bad_reply(source: Source, status: Option<u16>) -> RequestError {
    SourceError {
        source,
        reason: Reason::BadReply,
        status,
        tls_cause: None,
        retry_after: None,
    }
    .into()
}
