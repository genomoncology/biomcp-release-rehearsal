//! Binary payloads are atomic siblings of their owning exchange sequence.
use crate::RecordError;
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path},
};
const MAX_BINARY_BYTES: u64 = 50 * 1024 * 1024;
fn checked(parent: &Path, name: &str) -> Result<std::path::PathBuf, RecordError> {
    if name.len() > 128
        || !name.ends_with(".bin")
        || Path::new(name).components().count() != 1
        || !matches!(
            Path::new(name).components().next(),
            Some(Component::Normal(_))
        )
    {
        return Err(RecordError::Malformed);
    }
    for path in parent.ancestors() {
        if fs::symlink_metadata(path).is_ok_and(|v| v.file_type().is_symlink()) {
            return Err(RecordError::Malformed);
        }
    }
    Ok(parent.join(name))
}
pub(super) fn read(owner: &Path, name: &str, limit: u64) -> Result<Vec<u8>, RecordError> {
    let path = checked(
        owner
            .parent()
            .ok_or(RecordError::Io(std::io::ErrorKind::InvalidInput))?,
        name,
    )?;
    let meta = fs::symlink_metadata(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            RecordError::Missing
        } else {
            error.into()
        }
    })?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err(RecordError::Malformed);
    }
    let limit = limit.min(MAX_BINARY_BYTES);
    if meta.len() > limit {
        return Err(RecordError::Limit);
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(RecordError::from)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(RecordError::from)?;
    if bytes.len() as u64 > limit {
        return Err(RecordError::Limit);
    }
    Ok(bytes)
}
pub(super) fn publish(owner: &Path, index: usize, bytes: &[u8]) -> Result<String, RecordError> {
    if bytes.len() as u64 > MAX_BINARY_BYTES {
        return Err(RecordError::Limit);
    }
    let parent = owner
        .parent()
        .ok_or(RecordError::Io(std::io::ErrorKind::InvalidInput))?;
    let stem = owner
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or(RecordError::Malformed)?;
    let name = format!("{stem}-{index}.bin");
    let path = checked(parent, &name)?;
    fs::create_dir_all(parent).map_err(RecordError::from)?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(RecordError::from)?;
    file.write_all(bytes).map_err(RecordError::from)?;
    file.as_file().sync_all().map_err(RecordError::from)?;
    file.persist_noclobber(path)
        .map_err(|error| RecordError::from(error.error))?;
    Ok(name)
}
