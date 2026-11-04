//! Keep canonical provider spellings outside the transport implementation.
use super::{Check, rust_files};
use std::{fs, path::Path};

const EXCEPTIONS: &[&str] = &[];

pub(super) fn check(root: &Path) -> Check {
    let source = root.join("crates/biomcp-fetch/src");
    for path in rust_files(&source)? {
        let relative = path
            .strip_prefix(&source)?
            .to_string_lossy()
            .replace('\\', "/");
        if EXCEPTIONS.contains(&relative.as_str()) {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        for word in text.split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-') {
            if word.parse::<biomcp_types::Source>().is_ok() {
                return Err(format!(
                    "crates/biomcp-fetch/src/{relative}: transport names source {word}"
                )
                .into());
            }
        }
    }
    Ok(())
}
