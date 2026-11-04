//! Atomic explicit file saves shared by download commands. No saved-file registry exists.
pub(crate) mod article;
mod reply;
pub(crate) mod trial;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Checked explicit destination, without creating a file or directory.
#[derive(Debug)]
pub(crate) struct SaveTarget {
    path: PathBuf,
    replace: bool,
}
/// Receipt for the exact bytes written to an absolute destination.
#[derive(Debug)]
pub(crate) struct SavedFile {
    /// Absolute destination.
    pub path: PathBuf,
    /// Saved bytes.
    pub bytes: u64,
    /// Hexadecimal SHA-256.
    pub sha256: String,
}
impl SaveTarget {
    /// Resolve a file or existing folder and refuse clobbering before downloading.
    pub(crate) fn new(path: &str, filename: &str, replace: bool) -> io::Result<Self> {
        let input = Path::new(path);
        let path = if input.is_dir() {
            input.join(filename)
        } else {
            input.to_owned()
        };
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::other("missing filename"))?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let path = fs::canonicalize(parent)?.join(name);
        let encoded = path.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "destination is not UTF-8")
        })?;
        if biodata::Text::new(encoded).is_err() || encoded.trim() != encoded {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "destination cannot be represented",
            ));
        }
        let target = Self { path, replace };
        target.check()?;
        Ok(target)
    }
    fn check(&self) -> io::Result<()> {
        match fs::symlink_metadata(&self.path) {
            Ok(meta) if !self.replace || !meta.is_file() => Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination exists",
            )),
            Ok(_) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
    /// Write an adjacent temporary file, then atomically install it without partial files.
    pub(crate) fn write(self, bytes: &[u8]) -> io::Result<SavedFile> {
        self.check()?;
        let parent = self
            .path
            .parent()
            .ok_or_else(|| io::Error::other("missing parent"))?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        if self.replace {
            temporary.persist(&self.path).map_err(|error| error.error)?;
        } else {
            temporary
                .persist_noclobber(&self.path)
                .map_err(|error| error.error)?;
        }
        Ok(SavedFile {
            path: self.path,
            bytes: bytes.len() as u64,
            sha256: Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        })
    }
}

/// Keep destination policy and every filesystem mutation at the binary boundary.
pub(crate) fn run(
    cx: &biomcp_query::Ctx<'_>,
    query: &biomcp_types::ArticlePdf,
    allow_save: bool,
) -> biomcp_query::Outcome {
    let target = allow_save
        .then(|| {
            SaveTarget::new(
                query.save.as_str(),
                &biomcp_types::ArticlePdf::filename(&query.input),
                query.replace,
            )
        })
        .and_then(Result::ok);
    let download = biomcp_query::article::pdf::download(cx, query, target.is_some());
    let mut out = download.out;
    out.body = out.body.visit(reply::SaveReply {
        target,
        pdf: download.file,
        ..Default::default()
    });
    out
}
