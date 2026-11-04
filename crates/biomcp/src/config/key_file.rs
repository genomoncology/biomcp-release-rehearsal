//! Configuration persistence is confined to the settings boundary.
//! Key set/remove own these rereads under their write lock to preserve concurrent edits and field order.
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::Path,
};
mod order;
pub(crate) fn read(path: &Path) -> io::Result<Value> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(json!({})),
        Err(error) => Err(error),
    }
}
pub(crate) fn lock(path: &Path) -> io::Result<std::fs::File> {
    let target = write_target(path)?;
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut directory = std::fs::DirBuilder::new();
    directory.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(parent)?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(parent.join(".config.lock"))?;
    lock.lock()?;
    Ok(lock)
}
pub(crate) fn write(path: &Path, value: &Value) -> io::Result<()> {
    // Replace the selected target atomically, retaining dotfile-manager links.
    let target = write_target(path)?;
    let original = match std::fs::read_to_string(&target) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    let mut temporary = tempfile::NamedTempFile::new_in(
        target
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    serde_json::to_writer_pretty(&mut temporary, &order::Fields(value, original.as_deref()))?;
    temporary.write_all(b"\n")?;
    temporary.as_file().sync_all()?;
    temporary.persist(target).map_err(|e| e.error)?;
    Ok(())
}

fn write_target(path: &Path) -> io::Result<std::path::PathBuf> {
    let mut target = path.to_owned();
    for _ in 0..40 {
        match std::fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let link = std::fs::read_link(&target)?;
                target = target.parent().unwrap_or(Path::new(".")).join(link);
            }
            Ok(_) => return Ok(target),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(target),
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("configuration symlink chain is too long"))
}
