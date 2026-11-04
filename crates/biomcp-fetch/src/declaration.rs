use biomcp_types::Source;
use std::time::Duration;

/// A source declaration enforced by the transport.
#[derive(Debug)]
pub struct SourceSpec {
    /// Additional outbound destinations; the base URL declares the primary origin.
    pub destinations: crate::Destinations,
    /// Source-owned compatibility header; absent keeps the transport default.
    pub user_agent: Option<&'static str>,
    /// Source reported on failures.
    pub source: Source,
    /// Publisher-declared terms, reviewed by the source adapter.
    pub terms: Terms,
    /// Absolute base URL.
    pub base: &'static str,
    /// Credential policy.
    pub key: KeyUse,
    /// Provider limit shared with all sources using its key.
    pub quota: &'static crate::Quota,
    /// Maximum duration of this source's leg.
    pub deadline: Duration,
}

impl SourceSpec {
    /// Declare a JSON GET endpoint belonging to this source.
    pub const fn get(&'static self, path: &'static str) -> Endpoint {
        Endpoint {
            spec: self,
            path: std::borrow::Cow::Borrowed(path),
            fresh: Freshness::Never,
            method: Method::Get,
            idempotent: true,
            identity_encoding: false,
            octet_stream: None,
            publisher: false,
            byte_limit: None,
            json_rejections: false,
            json_absence: false,
            exact_count: false,
        }
    }
    /// Declare a HEAD endpoint that checks metadata without downloading its representation.
    pub const fn head(&'static self, path: &'static str) -> Endpoint {
        let mut endpoint = self.get(path);
        endpoint.method = Method::Head;
        endpoint
    }
    /// Declare a credential-free publisher download at a checked dynamic origin.
    /// Only a source with no credential policy can use this boundary.
    pub fn download(&'static self, url: &str) -> Result<Endpoint, crate::InputError> {
        if !matches!(self.key, KeyUse::None) {
            return Err(crate::InputError::Path);
        }
        let mut endpoint = self.get("");
        endpoint.path = std::borrow::Cow::Owned(crate::destination::checked(url)?);
        endpoint.publisher = true;
        Ok(endpoint)
    }
    /// Declare a form POST endpoint. Retries require explicit idempotence.
    pub const fn post_form(&'static self, path: &'static str) -> Endpoint {
        let mut endpoint = self.get(path);
        endpoint.method = Method::Form;
        endpoint.idempotent = false;
        endpoint
    }
    /// Declare a JSON POST endpoint. Retries require explicit idempotence.
    pub const fn post_json(&'static self, path: &'static str) -> Endpoint {
        let mut endpoint = self.get(path);
        endpoint.method = Method::Json;
        endpoint.idempotent = false;
        endpoint
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Method {
    Get,
    Head,
    Form,
    Json,
}

/// A declared endpoint, always attached to its source.
#[derive(Clone, Debug)]
pub struct Endpoint {
    pub(crate) spec: &'static SourceSpec,
    pub(crate) path: std::borrow::Cow<'static, str>,
    pub(crate) fresh: Freshness,
    pub(crate) method: Method,
    pub(crate) idempotent: bool,
    pub(crate) identity_encoding: bool,
    pub(crate) octet_stream: Option<&'static str>,
    pub(crate) publisher: bool,
    pub(crate) byte_limit: Option<u64>,
    pub(crate) json_rejections: bool,
    pub(crate) json_absence: bool,
    pub(crate) exact_count: bool,
}

/// Whether a source requires a credential before sending.
#[derive(Clone, Copy, Debug)]
pub enum KeyUse {
    /// No credential is sent.
    None,
    /// Send the credential when configured.
    Optional(Credential),
    /// Refuse all requests when the credential is absent.
    Required(Credential),
}

/// The credential name and its placement in an outgoing request.
#[derive(Clone, Copy, Debug)]
pub struct Credential {
    /// Whether hosted calls may fall back to the server credential.
    pub tier: biomcp_types::KeyTier,
    /// Configured secret to use.
    pub secret: SecretName,
    /// Where the provider expects its credential.
    pub place: KeyPlace,
}

/// Configuration names of supported secrets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecretName {
    /// openFDA API key.
    OpenfdaApiKey,
    /// NCBI API key.
    NcbiApiKey,
    /// Semantic Scholar API key.
    SemanticScholarApiKey,
    /// ORCID public access token.
    OrcidAccessToken,
    /// OncoKB API key.
    OncokbApiKey,
    /// AlphaGenome API key.
    AlphaGenomeApiKey,
    /// NCI Clinical Trials Search API key.
    NciApiKey,
}

/// Credential insertion location.
#[derive(Clone, Copy, Debug)]
pub enum KeyPlace {
    /// URL query field.
    Query(&'static str),
    /// HTTP request header.
    Header(&'static str),
    /// Authorization bearer token.
    Bearer,
}

/// A parameter value. Identifier sets are sorted before sending.
#[derive(Debug)]
pub enum Arg<'a> {
    /// Outgoing-only contact, excluded from persistent identities.
    Contact(&'a crate::Secret),
    /// Literal text.
    Text(&'a str),
    /// Unsigned integer.
    Num(u64),
    /// Identifier set joined by commas in lexical order.
    Ids(&'a [Box<str>]),
}

impl Arg<'_> {
    pub(crate) fn value(&self) -> String {
        match self {
            Self::Contact(_) => String::new(),
            Self::Text(value) => (*value).to_owned(),
            Self::Num(value) => value.to_string(),
            Self::Ids(values) => {
                let mut values: Vec<_> = values.iter().map(AsRef::as_ref).collect();
                values.sort_unstable();
                values.join(",")
            }
        }
    }
}

/// How long a checked endpoint reply remains fresh.
#[derive(Clone, Copy, Debug, Default)]
pub enum Freshness {
    /// Bypass all cache reads and writes.
    #[default]
    Never,
    /// Fresh for this many minutes.
    Minutes(u16),
    /// Fresh for this many hours.
    Hours(u16),
    /// Fresh for this many days.
    Days(u16),
}
impl Freshness {
    pub(crate) fn seconds(self) -> Option<u64> {
        match self {
            Self::Never => None,
            Self::Minutes(n) => Some(u64::from(n) * 60),
            Self::Hours(n) => Some(u64::from(n) * 3600),
            Self::Days(n) => Some(u64::from(n) * 86400),
        }
    }
}
impl Endpoint {
    /// Request the provider's exact count and retain its Content-Range metadata.
    pub const fn exact_count(mut self) -> Self {
        self.exact_count = true;
        self
    }

    /// Decode a checked JSON HTTP404 absence envelope without caching it.
    /// The reply type must distinguish exact source absence from an arbitrary route404.
    pub const fn json_absence(mut self) -> Self {
        self.json_absence = true;
        self
    }
    /// Decode typed JSON validation rejections (HTTP 400/422), never admitting them to cache.
    /// The reply type must check both success and rejection schemas. Other statuses retain normal retry semantics.
    pub const fn json_rejections(mut self) -> Self {
        self.json_rejections = true;
        self
    }
    /// Declare a bounded opaque binary reply independently of JSON/XML limits.
    pub const fn bytes(mut self, max_bytes: u64) -> Self {
        self.byte_limit = Some(max_bytes);
        self
    }
    /// Accept the reviewed bucket media type while retaining JSON/XML decoding and body limits.
    pub const fn octet_stream(mut self) -> Self {
        self.octet_stream = Some("binary/octet-stream");
        self
    }
    /// Admit the reviewed application/octet-stream media type while retaining typed decoding.
    pub const fn application_octet_stream(mut self) -> Self {
        self.octet_stream = Some("application/octet-stream");
        self
    }
    /// Request an uncompressed representation from a source with broken compression.
    pub const fn identity_encoding(mut self) -> Self {
        self.identity_encoding = true;
        self
    }
    /// Attempt this endpoint once, without retries for status or transport failures.
    pub const fn once(mut self) -> Self {
        self.idempotent = false;
        self
    }
    /// Permit retries for a POST that only reads provider data.
    pub const fn idempotent(mut self) -> Self {
        self.idempotent = true;
        self
    }
    /// Set the checked reply's cache freshness.
    pub const fn fresh(mut self, fresh: Freshness) -> Self {
        self.fresh = fresh;
        self
    }
}

impl Endpoint {
    /// Replace only the path, retaining the declared source, method and policy.
    /// Accept canonical absolute paths with unreserved ASCII segments only.
    pub fn with_path(mut self, path: impl Into<String>) -> Result<Self, crate::InputError> {
        let path = path.into();
        if !path.starts_with('/')
            || path.starts_with("//")
            || !path
                .bytes()
                .all(|ch| ch.is_ascii_alphanumeric() || b"/-._~".contains(&ch))
            || path.split('/').any(|part| matches!(part, "." | ".."))
        {
            return Err(crate::InputError::Path);
        }
        self.path = std::borrow::Cow::Owned(path);
        Ok(self)
    }
}

/// One raw identifier encoded as exactly one URL path segment.
#[derive(Clone, Debug)]
pub struct PathSegment(String);
impl PathSegment {
    /// Encode a raw identifier, rejecting pre-encoded input and path syntax.
    pub fn new(raw: &str) -> Result<Self, crate::InputError> {
        if raw.is_empty()
            || raw.split('/').any(|part| matches!(part, "." | ".."))
            || raw.starts_with("//")
            || raw.contains("://")
            || raw
                .chars()
                .any(|c| c.is_control() || matches!(c, '\\' | '%'))
        {
            return Err(crate::InputError::Path);
        }
        let mut encoded = String::new();
        for byte in raw.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                encoded.push(char::from(byte));
            } else {
                use std::fmt::Write;
                write!(encoded, "%{byte:02X}").map_err(|_| crate::InputError::Path)?;
            }
        }
        Ok(Self(encoded))
    }
}
impl Endpoint {
    /// Append checked raw segments to this declared endpoint without changing its origin.
    pub fn with_segments(mut self, segments: &[PathSegment]) -> Self {
        let mut path = self.path.trim_end_matches('/').to_owned();
        for segment in segments {
            path.push('/');
            path.push_str(&segment.0);
        }
        self.path = std::borrow::Cow::Owned(path);
        self
    }
}
/// Publisher evidence for a source's reuse category; not a grant of rights.
#[derive(Clone, Copy, Debug)]
pub struct Terms {
    /// Publisher license or terms name.
    pub name: &'static str,
    /// Publisher page documenting the terms.
    pub url: &'static str,
    /// Reviewed category used by the source switch.
    pub usage: biomcp_types::Use,
}
