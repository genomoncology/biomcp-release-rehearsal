//! Durable test evidence shares the checkout default and explicit override.
#![expect(
    clippy::disallowed_methods,
    reason = "The fixture owner selects durable test evidence outside scheduler scratch"
)]
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Keep installed-runner evidence beside the copied fixtures, outside child scratch.
pub fn default_directory() -> PathBuf {
    std::env::var_os("BIOMCP_TEST_FIXTURES")
        .and_then(|fixtures| PathBuf::from(fixtures).parent().map(Path::to_owned))
        .map_or_else(
            || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fixture-evidence"),
            |scratch| scratch.join("target/fixture-evidence"),
        )
}

/// Create a retained evidence directory outside disposable test scratch.
pub fn directory() -> io::Result<PathBuf> {
    let fallback = default_directory();
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
