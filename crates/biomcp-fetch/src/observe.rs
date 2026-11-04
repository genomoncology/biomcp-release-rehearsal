//! Opt-in bounded wire and bundle-wait summaries with credential-free request URLs.
use serde::{Deserialize, Serialize};
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded transport state across the request owners named by this module"
)]
use std::{collections::BTreeMap, fs::File, io::Write, path::Path, sync::Mutex};
/// Maximum attempts retained per command.
pub const MAX_OBSERVATIONS: usize = 1024;
const MAX_REPORT_BYTES: usize = 8 * 1024 * 1024;
const FIELDS: usize = 64;
/// Raw structure before adapter mapping.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Shape {
    /// Sampled JSON paths or XML names.
    pub fields: BTreeMap<String, bool>,
    /// Array or declared count is positive.
    pub positive: BTreeMap<String, bool>,
    /// Structure exceeded a sampling bound or could not be inspected completely.
    #[serde(default = "unknown_shape")]
    pub incomplete: bool,
    /// Bounded tabular header row.
    pub header: Option<String>,
}
fn unknown_shape() -> bool {
    true
}
impl Shape {
    /// Inspect structure without retaining text or scalar values.
    pub fn inspect(body: &[u8], media: &str) -> Self {
        let mut shape = Self::default();
        if media.contains("json") {
            if let Ok(value) = serde_json::from_slice(body) {
                shape.json(&value, "", 0);
            } else {
                shape.incomplete = true;
            }
        } else if media.contains("xml") {
            shape.xml(body);
        } else if media.contains("csv") || media.contains("tab-separated") {
            shape.incomplete = body
                .split(|b| *b == b'\n')
                .next()
                .is_some_and(|line| line.len() > 4096);
            shape.header = body
                .split(|b| *b == b'\n')
                .next()
                .filter(|line| line.len() <= 4096)
                .and_then(|line| std::str::from_utf8(line).ok())
                .map(str::to_owned);
        }
        shape
    }
    fn xml(&mut self, body: &[u8]) {
        let mut reader = quick_xml::Reader::from_reader(body);
        let mut count = None;
        while self.fields.len() < FIELDS {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Start(e) | quick_xml::events::Event::Empty(e)) => {
                    let name = e.name().as_ref().to_owned();
                    count = name
                        .to_ascii_lowercase()
                        .contains("count")
                        .then(|| name.clone());
                    if name.len() <= 128 {
                        self.fields.insert(name, true);
                    } else {
                        self.incomplete = true;
                    }
                }
                Ok(quick_xml::events::Event::Text(text)) => {
                    if let Some(name) = count.take()
                        && let Ok(value) = text.as_ref().trim().parse::<u64>()
                    {
                        self.positive.insert(name, value > 0);
                    }
                }
                Ok(quick_xml::events::Event::Eof) => return,
                Err(_) => {
                    self.incomplete = true;
                    return;
                }
                Ok(quick_xml::events::Event::End(_)) => count = None,
                _ => {}
            }
        }
        self.incomplete = true;
    }
    fn json(&mut self, value: &serde_json::Value, path: &str, depth: usize) {
        if depth >= 8 || path.len() > 256 || self.fields.len() >= FIELDS {
            self.incomplete = true;
            return;
        }
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if key.len() > 128 || self.fields.len() >= FIELDS {
                        self.incomplete = true;
                        break;
                    }
                    let segment = if key
                        .as_bytes()
                        .first()
                        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
                        && key
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
                    {
                        key.as_str()
                    } else {
                        "*"
                    };
                    let path = format!("{path}/{segment}");
                    self.fields.insert(path.clone(), true);
                    if key.to_ascii_lowercase().contains("count")
                        && let Some(n) = value.as_u64().or_else(|| value.as_str()?.parse().ok())
                    {
                        self.positive.insert(path.clone(), n > 0);
                    }
                    self.json(value, &path, depth + 1);
                }
            }
            serde_json::Value::Array(items) => {
                self.incomplete |= items.len() > 3;
                self.positive.insert(path.to_owned(), !items.is_empty());
                for item in items.iter().take(3) {
                    self.json(item, &format!("{path}/*"), depth + 1);
                }
            }
            _ => {}
        }
    }
}
/// One attempt, including failed and retried attempts.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Observation {
    /// Transport's prepared credential-free identity.
    pub identity: String,
    /// Prepared URL, before any credential or contact insertion.
    #[serde(default)]
    pub url: String,
    /// Checked reply type or bundle-wait entry, without request values.
    #[serde(default)]
    pub role: String,
    /// Reused checked local data rather than a transport attempt.
    #[serde(default)]
    pub cached: bool,
    /// Provider source.
    pub source: biomcp_types::Source,
    /// HTTP status, absent on transport failure.
    pub status: Option<u16>,
    /// Safe TLS failure cause; no certificate or library detail is retained.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tls_cause: Option<crate::TlsCause>,
    /// Retained decoded body bytes; transport/read failures retain no body.
    pub bytes: u64,
    /// Nominal keyless interval from the same shared quota declaration.
    pub interval_ms: u64,
    /// Wire duration in milliseconds.
    pub elapsed_ms: u64,
    /// Optional explicit discovery and dependent request context.
    #[serde(default)]
    pub provenance: crate::Provenance,
    /// Bounded raw response shape.
    pub shape: Shape,
}
/// Explicitly created command-owned sink.
#[derive(Debug)]
#[expect(
    clippy::disallowed_types,
    reason = "Serialize bounded opt-in summaries across source workers"
)]
pub struct Observations(Mutex<(File, usize, usize)>);
impl Observations {
    #[expect(
        clippy::disallowed_types,
        reason = "Serialize bounded opt-in summaries across source workers"
    )]
    /// Create exclusively; never overwrite an earlier report.
    pub fn create(path: &Path) -> std::io::Result<Self> {
        Ok(Self(Mutex::new((
            File::options().write(true).create_new(true).open(path)?,
            0,
            0,
        ))))
    }
    pub(crate) fn push(&self, item: &Observation) {
        let mut state = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.1 > MAX_OBSERVATIONS {
            return;
        }
        if let Ok(mut line) = serde_json::to_vec(item) {
            line.push(b'\n');
            if state.1 == MAX_OBSERVATIONS || state.2.saturating_add(line.len()) > MAX_REPORT_BYTES
            {
                let _ = state
                    .0
                    .write_all(b"{\"error\":\"observation limit exceeded\"}\n");
                state.1 = MAX_OBSERVATIONS + 1;
                return;
            }
            state.1 += 1;
            state.2 += line.len();
            let _ = state.0.write_all(&line);
        }
    }
}
impl crate::Leg<'_> {
    pub(crate) fn observe(
        &self,
        request: (&[u8; 32], &str),
        status: Option<u16>,
        body: &[u8],
        media: &str,
        elapsed: std::time::Duration,
    ) {
        if self.fetch.config.observations.is_some() {
            self.observed(
                request.0,
                status,
                body.len() as u64,
                Shape::inspect(body, media),
                elapsed,
                (
                    "bundle",
                    false,
                    request.1,
                    crate::Provenance::default(),
                    None,
                ),
            );
        }
    }
    pub(crate) fn observe_tls_failure(
        &self,
        request: (&[u8; 32], &str),
        elapsed: std::time::Duration,
        error: &ureq::Error,
    ) {
        self.observed(
            request.0,
            None,
            0,
            Shape::default(),
            elapsed,
            (
                "bundle",
                false,
                request.1,
                crate::Provenance::default(),
                crate::tls::cause(error),
            ),
        );
    }
    pub(crate) fn observe_bundle(
        &self,
        request: (&[u8; 32], &str),
        status: Option<u16>,
        body: &[u8],
        format: crate::RowFormat,
        elapsed: std::time::Duration,
    ) {
        if self.fetch.config.observations.is_none() {
            return;
        }
        let banner = match format {
            crate::RowFormat::Document { .. }
            | crate::RowFormat::Tsv
            | crate::RowFormat::NativeTsv { .. }
            | crate::RowFormat::Json { .. }
            | crate::RowFormat::NativeCsv { .. } => 0,
            crate::RowFormat::Csv { banner_lines } => banner_lines,
        };
        let mut header = body;
        for _ in 0..banner {
            header = header.splitn(2, |b| *b == b'\n').nth(1).unwrap_or(&[]);
        }
        let media = format.media();
        self.observed(
            request.0,
            status,
            body.len() as u64,
            Shape::inspect(header, media),
            elapsed,
            (
                "bundle",
                false,
                request.1,
                crate::Provenance::default(),
                None,
            ),
        );
    }
    pub(crate) fn observed(
        &self,
        identity: &[u8; 32],
        status: Option<u16>,
        bytes: u64,
        shape: Shape,
        elapsed: std::time::Duration,
        context: (&str, bool, &str, crate::Provenance, Option<crate::TlsCause>),
    ) {
        if let Some(sink) = &self.fetch.config.observations {
            sink.push(&Observation {
                identity: identity.iter().map(|byte| format!("{byte:02x}")).collect(),
                url: context.2.to_owned(),
                role: context.0.to_owned(),
                cached: context.1,
                source: self.spec.source,
                status,
                bytes,
                interval_ms: u64::try_from(
                    self.spec
                        .quota
                        .interval(false)
                        .as_nanos()
                        .div_ceil(1_000_000),
                )
                .unwrap_or(u64::MAX),
                elapsed_ms: u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
                shape,
                provenance: context.3,
                tls_cause: context.4,
            });
        }
    }
}

/// Record a SQLite error without changing its classification or the command result.
#[track_caller]
#[expect(
    clippy::disallowed_methods,
    reason = "The proof diagnostic boundary alone reads its opt-in destination"
)]
pub fn record_sqlite_error(component: &'static str, error: &rusqlite::Error) {
    let Some(path) = std::env::var_os(crate::SQLITE_DIAGNOSTICS) else {
        return;
    };
    let caller = std::panic::Location::caller();
    // Only a compiled source basename is retained, never a runtime path or SQL.
    let source = caller
        .file()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("unknown");
    let operation = format!("{component}::{source}:{}", caller.line());
    crate::sqlite_diagnostic::append(Path::new(&path), &operation, error);
}
