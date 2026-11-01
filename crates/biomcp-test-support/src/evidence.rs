//! Durable test evidence shares the checkout default and explicit override.
#![expect(
    clippy::disallowed_methods,
    reason = "The fixture owner selects durable test evidence outside scheduler scratch"
)]
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Create a retained evidence directory outside disposable test scratch.
pub fn directory() -> io::Result<PathBuf> {
    let fallback = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fixture-evidence");
    let directory = std::env::var_os("BIOMCP_EVIDENCE_DIR").map_or_else(
        || fallback.clone(),
        |p| PathBuf::from(p).join("fixture-failures"),
    );
    fs::create_dir_all(&directory)?;
    let directory = directory.canonicalize()?;
    // A configured temporary directory cannot defeat native-gate retention.
    if std::env::var_os("TMPDIR")
        .map(PathBuf::from)
        .and_then(|p| p.canonicalize().ok())
        .is_some_and(|p| directory.starts_with(p))
    {
        fs::create_dir_all(&fallback)?;
        return fallback.canonicalize();
    }
    Ok(directory)
}
