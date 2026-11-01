//! Bounded, credential-free captures and atomically consumed replay sequences.
use crate::{CacheError, Mode, RecordError, cache::Cache};
use serde::{Deserialize, Serialize};
#[expect(
    clippy::disallowed_types,
    reason = "Share bounded transport state across the request owners named by this module"
)]
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

#[path = "record_rpc.rs"]
pub(crate) mod rpc;
#[path = "record_sidecar.rs"]
mod sidecar;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_BUNDLE_FILE_BYTES: u64 = crate::MAX_BUNDLE_BYTES * 6 + 64 * 1024;
const MAX_METADATA_BYTES: usize = 8192;
const MAX_EXCHANGES: usize = 128;
const LOCK: &str = ".record-owner";

/// Request identity metadata keeps GET and HEAD captures separate without POST fields.
#[derive(Clone, Copy)]
pub(crate) enum Request<'a> {
    Get,
    Head,
    Post(&'a crate::prepared::PostMetadata),
}
impl<'a> From<Option<&'a crate::prepared::PostMetadata>> for Request<'a> {
    fn from(post: Option<&'a crate::prepared::PostMetadata>) -> Self {
        post.map_or(Self::Get, Self::Post)
    }
}
impl<'a> Request<'a> {
    fn method(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post(_) => "POST",
        }
    }
    fn post(self) -> Option<&'a crate::prepared::PostMetadata> {
        match self {
            Self::Post(post) => Some(post),
            _ => None,
        }
    }
}

/// An opaque temporary cache shared by one replay case and its offline checks.
#[derive(Clone)]
pub struct ReplayCache(Arc<CaseCache>);
#[derive(Debug)]
struct CaseCache {
    cache: Arc<Cache>,
    _directory: Option<tempfile::TempDir>,
}
impl std::fmt::Debug for ReplayCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReplayCache")
    }
}
impl ReplayCache {
    /// Allocate an isolated cache whose files live until its last owner drops.
    pub fn temporary() -> Result<Self, CacheError> {
        Self::temporary_with_owners(&[])
    }
    /// Allocate a cache while checking all declared schema owners first.
    pub fn temporary_with_owners(owners: &[crate::SchemaOwner]) -> Result<Self, CacheError> {
        let directory = tempfile::tempdir().map_err(|_| CacheError::Filesystem)?;
        let cache = Arc::new(Cache::open_with_owners(
            &directory.path().join("replay.sqlite"),
            owners,
        )?);
        Ok(Self(Arc::new(CaseCache {
            cache,
            _directory: Some(directory),
        })))
    }
    /// Create a marked case with all schema owners checked before mutation.
    pub fn create_case_with_owners(
        path: &Path,
        owners: &[crate::SchemaOwner],
    ) -> Result<Self, CacheError> {
        safe_path(path)?;
        fs::create_dir(path).map_err(|_| CacheError::Filesystem)?;
        let result = (|| {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join(".biomcp-case"))?
                .write_all(b"biomcp.replay-case/1\n")?;
            Ok::<_, std::io::Error>(())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(path);
            return Err(CacheError::Filesystem);
        }
        let cache = Arc::new(Cache::open_with_owners(
            &path.join("replay.sqlite"),
            owners,
        )?);
        Ok(Self(Arc::new(CaseCache {
            cache,
            _directory: None,
        })))
    }
    /// Open an owned case after checking every declared schema owner.
    pub fn open_case_with_owners(
        path: &Path,
        owners: &[crate::SchemaOwner],
    ) -> Result<Self, CacheError> {
        safe_path(path)?;
        if !fs::symlink_metadata(path).is_ok_and(|meta| meta.is_dir()) {
            return Err(CacheError::Version);
        }
        for entry in fs::read_dir(path).map_err(|_| CacheError::Filesystem)? {
            let entry = entry.map_err(|_| CacheError::Filesystem)?;
            let name = entry.file_name();
            if ![
                ".biomcp-case",
                "replay.sqlite",
                "replay.sqlite-wal",
                "replay.sqlite-shm",
            ]
            .iter()
            .any(|allowed| name == *allowed)
                || !entry
                    .file_type()
                    .map_err(|_| CacheError::Filesystem)?
                    .is_file()
            {
                return Err(CacheError::Version);
            }
        }
        if fs::read(path.join(".biomcp-case")).map_err(|_| CacheError::Version)?
            != b"biomcp.replay-case/1\n"
            || !path.join("replay.sqlite").is_file()
        {
            return Err(CacheError::Version);
        }
        let cache = Arc::new(Cache::open_with_owners(
            &path.join("replay.sqlite"),
            owners,
        )?);
        Ok(Self(Arc::new(CaseCache {
            cache,
            _directory: None,
        })))
    }
    pub(crate) fn cache(&self) -> Arc<Cache> {
        self.0.cache.clone()
    }
}

fn safe_path(path: &Path) -> Result<(), CacheError> {
    for ancestor in path.ancestors() {
        if ancestor
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(CacheError::Version);
        }
        match fs::symlink_metadata(ancestor) {
            Ok(meta) if meta.file_type().is_symlink() => return Err(CacheError::Version),
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                return Err(CacheError::Filesystem);
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Recording {
    method: String,
    source: biomcp_types::Source,
    url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    post: Option<crate::prepared::PostMetadata>,
    exchanges: Vec<Exchange>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Exchange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) rpc: Option<rpc::Rpc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) fetched: Option<i64>,
    pub(crate) status: u16,
    pub(crate) content_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) content_range: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pagination: Option<crate::Pagination>,
    pub(crate) delay_ms: u64,
    pub(crate) retry_after_ms: Option<u64>,
    pub(crate) body: Body,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(crate) enum Body {
    Json(Box<serde_json::value::RawValue>),
    Text(String),
    Sidecar(String),
    #[serde(skip)]
    Bytes(Vec<u8>),
}
impl Body {
    pub(crate) fn capture(bytes: Vec<u8>) -> Result<Self, RecordError> {
        Self::capture_bounded(bytes, MAX_FILE_BYTES)
    }
    pub(crate) fn capture_bundle(bytes: Vec<u8>) -> Result<Self, RecordError> {
        if bytes.len() as u64 > crate::MAX_BUNDLE_BYTES {
            return Err(RecordError::Limit);
        }
        Ok(Self::Text(
            String::from_utf8(bytes).map_err(|_| RecordError::NonUtf8)?,
        ))
    }
    fn capture_bounded(bytes: Vec<u8>, limit: u64) -> Result<Self, RecordError> {
        if bytes.len() as u64 > limit {
            return Err(RecordError::Limit);
        }
        let text = String::from_utf8(bytes).map_err(|_| RecordError::NonUtf8)?;
        Ok(
            match serde_json::value::RawValue::from_string(text.clone()) {
                Ok(value) => Self::Json(value),
                Err(_) => Self::Text(text),
            },
        )
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        match self {
            Self::Json(value) => value.get().as_bytes(),
            Self::Text(text) => text.as_bytes(),
            Self::Bytes(bytes) => bytes,
            Self::Sidecar(_) => &[],
        }
    }
}
impl Exchange {
    fn valid(&self) -> bool {
        self.rpc.as_ref().is_none_or(rpc::Rpc::valid)
            && (100..=599).contains(&self.status)
            && self
                .pagination
                .as_ref()
                .is_none_or(crate::Pagination::valid)
            && self.content_range.as_ref().is_none_or(|v| {
                v.len() <= MAX_METADATA_BYTES
                    && v.bytes().all(|b| b.is_ascii() && !b.is_ascii_control())
            })
            && self.content_type.len() <= MAX_METADATA_BYTES
            && self
                .content_type
                .bytes()
                .all(|b| b.is_ascii() && !b.is_ascii_control())
    }
}

#[derive(Debug)]
#[expect(
    clippy::disallowed_types,
    reason = "Reserve replay cursors and publish captures across scoped callers"
)]
pub(crate) enum Recordings {
    None,
    Capture(Capture),
    Replay {
        dir: PathBuf,
        cursors: Mutex<BTreeMap<(biomcp_types::Source, [u8; 32]), Cursor>>,
    },
}
#[derive(Debug)]
#[expect(
    clippy::disallowed_types,
    reason = "Reserve replay cursors and publish captures across scoped callers"
)]
pub(crate) struct Capture {
    dir: PathBuf,
    sequences: Mutex<BTreeMap<(biomcp_types::Source, [u8; 32]), Recording>>,
}
#[derive(Debug)]
pub(crate) struct Cursor {
    recording: Recording,
    file_bytes: u64,
    next: usize,
}
impl Drop for Capture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.dir.join(LOCK));
    }
}
impl Recordings {
    #[expect(
        clippy::disallowed_types,
        reason = "Reserve replay cursors and publish captures across scoped callers"
    )]
    pub(crate) fn open(mode: &Mode) -> Result<Self, RecordError> {
        match mode {
            Mode::Record { dir } => {
                if let Err(error) = fs::create_dir(dir)
                    && error.kind() != std::io::ErrorKind::AlreadyExists
                {
                    return Err(error.into());
                }
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(dir.join(LOCK))
                    .map_err(|error| {
                        if error.kind() == std::io::ErrorKind::AlreadyExists {
                            RecordError::Conflict
                        } else {
                            error.into()
                        }
                    })?;
                let capture = Capture {
                    dir: dir.clone(),
                    sequences: Mutex::new(BTreeMap::new()),
                };
                for entry in fs::read_dir(dir).map_err(RecordError::from)? {
                    if entry.map_err(RecordError::from)?.file_name() != LOCK {
                        return Err(RecordError::Conflict);
                    }
                }
                Ok(Self::Capture(capture))
            }
            Mode::Replay { dir } => Ok(Self::Replay {
                dir: dir.clone(),
                cursors: Mutex::new(BTreeMap::new()),
            }),
            _ => Ok(Self::None),
        }
    }
    pub(crate) fn replay(
        &self,
        source: biomcp_types::Source,
        key: &[u8; 32],
        url: &str,
        request: Request<'_>,
        max_body: u64,
    ) -> Result<Option<Exchange>, RecordError> {
        self.replay_bounded(source, key, url, request, max_body, MAX_FILE_BYTES)
    }
    pub(crate) fn replay_bundle(
        &self,
        source: biomcp_types::Source,
        key: &[u8; 32],
        url: &str,
    ) -> Result<Option<Exchange>, RecordError> {
        self.replay_bounded(
            source,
            key,
            url,
            Request::Get,
            crate::MAX_BUNDLE_BYTES,
            MAX_BUNDLE_FILE_BYTES,
        )
    }
    fn replay_bounded(
        &self,
        source: biomcp_types::Source,
        key: &[u8; 32],
        url: &str,
        request: Request<'_>,
        max_body: u64,
        file_limit: u64,
    ) -> Result<Option<Exchange>, RecordError> {
        let Self::Replay { dir, cursors } = self else {
            return Ok(None);
        };
        let mut cursors = cursors
            .lock()
            .map_err(|_| RecordError::Io(std::io::ErrorKind::Other))?;
        let cursor = match cursors.entry((source, *key)) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let (recording, file_bytes) =
                    read(&path(dir, source, key), source, url, request, file_limit)?;
                entry.insert(Cursor {
                    recording,
                    file_bytes,
                    next: 0,
                })
            }
        };
        if cursor.file_bytes > file_limit {
            return Err(RecordError::Limit);
        }
        let value = cursor
            .recording
            .exchanges
            .get(cursor.next)
            .ok_or(RecordError::Exhausted)?;
        let mut value = value.clone();
        if let Body::Sidecar(name) = &value.body {
            value.body = Body::Bytes(sidecar::read(&path(dir, source, key), name, max_body)?);
        }
        if matches!(request, Request::Head) && !value.body.bytes().is_empty() {
            return Err(RecordError::Malformed);
        }
        cursor.next += 1;
        if value.body.bytes().len() as u64 > max_body {
            return Err(RecordError::Limit);
        }
        Ok(Some(value.clone()))
    }
    pub(crate) fn append(
        &self,
        source: biomcp_types::Source,
        key: &[u8; 32],
        url: &str,
        request: Request<'_>,
        exchange: Exchange,
    ) -> Result<(), RecordError> {
        self.append_bounded(source, key, url, request, exchange, MAX_FILE_BYTES)
    }
    pub(crate) fn append_bundle(
        &self,
        source: biomcp_types::Source,
        key: &[u8; 32],
        url: &str,
        exchange: Exchange,
    ) -> Result<(), RecordError> {
        self.append_bounded(
            source,
            key,
            url,
            Request::Get,
            exchange,
            MAX_BUNDLE_FILE_BYTES,
        )
    }
    fn append_bounded(
        &self,
        source: biomcp_types::Source,
        key: &[u8; 32],
        url: &str,
        request: Request<'_>,
        exchange: Exchange,
        limit: u64,
    ) -> Result<(), RecordError> {
        let Self::Capture(capture) = self else {
            return Ok(());
        };
        if url.len() > MAX_METADATA_BYTES || !exchange.valid() {
            return Err(RecordError::Limit);
        }
        let mut sequences = capture
            .sequences
            .lock()
            .map_err(|_| RecordError::Io(std::io::ErrorKind::Other))?;
        let recording = sequences
            .entry((source, *key))
            .or_insert_with(|| Recording {
                method: request.method().into(),
                post: request.post().cloned(),
                source,
                url: url.into(),
                exchanges: Vec::new(),
            });
        if recording.exchanges.len() >= MAX_EXCHANGES {
            return Err(RecordError::Limit);
        }
        let mut exchange = exchange;
        if let Body::Bytes(bytes) = &exchange.body {
            let name = sidecar::publish(
                &path(&capture.dir, source, key),
                recording.exchanges.len(),
                bytes,
            )?;
            exchange.body = Body::Sidecar(name);
        }
        recording.exchanges.push(exchange);
        let result = publish(&path(&capture.dir, source, key), recording, limit);
        if result.is_err() {
            recording.exchanges.pop();
        }
        result
    }
}
fn path(dir: &Path, source: biomcp_types::Source, key: &[u8; 32]) -> PathBuf {
    let key: String = key.iter().map(|byte| format!("{byte:02x}")).collect();
    dir.join(source.to_string()).join(format!("{key}.json"))
}
fn read(
    path: &Path,
    source: biomcp_types::Source,
    url: &str,
    request: Request<'_>,
    limit: u64,
) -> Result<(Recording, u64), RecordError> {
    let file = fs::File::open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            RecordError::Missing
        } else {
            error.into()
        }
    })?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(RecordError::from)?;
    if bytes.len() as u64 > limit {
        return Err(RecordError::Limit);
    }
    let value: Recording = serde_json::from_slice(&bytes).map_err(|_| RecordError::Malformed)?;
    if value.method != request.method()
        || value.post.as_ref() != request.post()
        || value.source != source
        || value.url != url
        || value.url.len() > MAX_METADATA_BYTES
        || value.exchanges.is_empty()
        || value.exchanges.len() > MAX_EXCHANGES
        || value.exchanges.iter().any(|exchange| !exchange.valid())
    {
        return Err(RecordError::Malformed);
    }
    Ok((value, bytes.len() as u64))
}
fn publish(path: &Path, value: &Recording, limit: u64) -> Result<(), RecordError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| RecordError::Malformed)?;
    if bytes.len() as u64 > limit {
        return Err(RecordError::Limit);
    }
    let parent = path
        .parent()
        .ok_or(RecordError::Io(std::io::ErrorKind::InvalidInput))?;
    fs::create_dir_all(parent).map_err(RecordError::from)?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(RecordError::from)?;
    file.write_all(&bytes).map_err(RecordError::from)?;
    file.as_file().sync_all().map_err(RecordError::from)?;
    file.persist(path)
        .map_err(|error| RecordError::from(error.error))?;
    Ok(())
}
pub(crate) fn millis(duration: Duration) -> Result<u64, RecordError> {
    u64::try_from(duration.as_millis()).map_err(|_| RecordError::Limit)
}
