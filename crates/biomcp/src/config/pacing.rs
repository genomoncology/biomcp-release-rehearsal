//! Merge operator overrides once before opening shared source transports.
use super::{
    ConfigError, Globals,
    file::{self, ConfigFile},
};

pub(super) fn resolve(
    globals: &Globals,
    file: Option<&ConfigFile>,
) -> Result<biomcp_fetch::SourcePacing, ConfigError> {
    let mut values = file
        .and_then(|f| f.source_pacing.clone())
        .unwrap_or_default();
    if let Some(text) = &globals.source_pacing {
        let input = serde_json::from_str(text).map_err(|_| ConfigError::field("source_pacing"))?;
        for (name, flags) in file::pacing(input)? {
            let merged = values.get(&name).copied().unwrap_or_default().merge(flags);
            values.insert(name, merged);
        }
    }
    biomcp_query::sources::source_pacing(&values).map_err(|name| ConfigError::field(&name))
}
