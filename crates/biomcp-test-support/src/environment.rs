//! Isolate child home, config, cache and credentials while retaining OS and coverage settings.
#![expect(
    clippy::disallowed_methods,
    reason = "The shared test launcher owns child environment and executable selection"
)]
use std::{fs, io, path::Path, process::Command};
/// Child environment paths rooted in each isolated test environment.
pub const ENV_PATHS: &[(&str, &str)] = &[
    ("HOME", "home"),
    ("USERPROFILE", "home"),
    ("APPDATA", "config"),
    ("LOCALAPPDATA", "cache"),
    ("XDG_CONFIG_HOME", "config"),
    ("XDG_CACHE_HOME", "cache"),
];
/// Create an isolated temporary environment.
pub fn new() -> io::Result<tempfile::TempDir> {
    let root = tempfile::Builder::new().prefix("biomcp-").tempdir()?;
    for directory in ["home", "config", "cache"] {
        fs::create_dir(root.path().join(directory))?;
    }
    Ok(root)
}
/// Launch a child with isolated home, config, cache and credentials.
pub fn command(program: &Path, root: &Path, args: &[String]) -> Command {
    let mut command = Command::new(program);
    command
        .args(args)
        .env_clear()
        .envs(ENV_PATHS.iter().map(|(key, path)| (key, root.join(path))))
        .current_dir(root);
    // Winsock needs SystemRoot. Child-only product coverage uses the parent's profile pattern.
    // The load test's SQLite notes come from the product child, not the test runner.
    for name in [
        "SystemRoot",
        "LLVM_PROFILE_FILE",
        "BIOMCP_SQLITE_DIAGNOSTICS",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}
