//! MCP credentials reload for each call while all static settings keep their startup snapshot.
use super::{ConfigError, Globals, Secrets};

pub(crate) fn refresh(globals: &Globals) -> Result<Secrets, ConfigError> {
    let file = super::file::read_file(globals.config.as_ref())?;
    Ok(super::secrets_from(file.as_ref()))
}
