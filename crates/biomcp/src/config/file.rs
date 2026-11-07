//! Configuration decoding retains field names without retaining rejected values.
use super::config_path;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, fs, path::PathBuf};

/// Keep both copies until the deployed clients have been checked for structured output.
#[derive(Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReplyForm {
    #[default]
    Both,
    StructuredOnly,
    TextOnly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ToolMode {
    #[default]
    Single,
    Typed,
}

#[derive(Clone, Debug)]
pub(crate) struct ConfigError(pub(crate) String, ConfigScope);
#[derive(Clone, Debug)]
enum ConfigScope {
    SourcePacing,
    Other,
}
impl ConfigError {
    pub(super) fn is_pacing(&self) -> bool {
        matches!(self.1, ConfigScope::SourcePacing)
    }
    pub(crate) fn field(name: &str) -> Self {
        // An unknown key is a name, but arbitrary JSON strings are not safe diagnostics.
        let safe = !name.is_empty()
            && name.len() <= 96
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.' || b == b'-');
        let scope = if name == "source_pacing" || name.starts_with("source_pacing.") {
            ConfigScope::SourcePacing
        } else {
            ConfigScope::Other
        };
        Self(
            if safe {
                name.into()
            } else {
                "config.json".into()
            },
            scope,
        )
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ConfigFile {
    pub(super) hosted_max_use: Option<crate::command::DeclaredUse>,
    pub(super) mcp_tool_mode: Option<ToolMode>,
    pub(super) mcp_reply_form: Option<ReplyForm>,
    pub(super) mcp_typed_output_schema: Option<bool>,
    pub(super) mcp_typed_enum_limit: Option<usize>,
    pub(super) mcp_typed_tool_limit: Option<std::num::NonZeroUsize>,
    pub(super) mcp_typed_definition_bytes: Option<std::num::NonZeroUsize>,
    pub(super) format: Option<crate::grammar::Format>,
    pub(super) proxy: Option<String>,
    pub(super) http: Option<Http>,
    pub(super) disable_sources: Option<Vec<String>>,
    pub(super) ca_bundle: Option<PathBuf>,
    pub(super) ncbi_email: Option<String>,
    pub(super) keys: Option<BTreeMap<String, String>>,
    pub(super) auto_sync_max_bytes: Option<u64>,
    pub(super) command_timeout_seconds: Option<std::num::NonZeroU32>,
    pub(super) drug_approvals_limit: Option<u32>,
    pub(super) drug_search_name_limit: Option<std::num::NonZeroUsize>,
    pub(super) source_pacing: Option<BTreeMap<String, biomcp_fetch::PacingOverride>>,
    pub(super) http_max_calls: Option<std::num::NonZeroUsize>,
    pub(super) http_max_connections: Option<std::num::NonZeroUsize>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Http {
    pub(super) allowed_hosts: Vec<String>,
}
fn take<T: DeserializeOwned>(
    object: &mut Map<String, Value>,
    name: &str,
) -> Result<Option<T>, ConfigError> {
    object
        .remove(name)
        .filter(|value| !value.is_null())
        .map(|value| serde_json::from_value(value).map_err(|_| ConfigError::field(name)))
        .transpose()
}
fn object(value: Value, name: &str) -> Result<Map<String, Value>, ConfigError> {
    match value {
        Value::Object(map) => Ok(map),
        _ => Err(ConfigError::field(name)),
    }
}
fn unknown(map: &Map<String, Value>, prefix: &str) -> Result<(), ConfigError> {
    if let Some(key) = map.keys().next() {
        return Err(ConfigError::field(&format!("{prefix}{key}")));
    }
    Ok(())
}
fn decode(bytes: &[u8]) -> Result<ConfigFile, ConfigError> {
    let file: ConfigFile = serde_json::from_slice(bytes).map_err(|error| {
        // Only a rejected document needs field diagnosis. Retain serde's duplicate-field
        // refusal while ensuring its value-bearing error message never escapes.
        diagnose(bytes).err().unwrap_or_else(|| {
            let message = error.to_string();
            let name = message
                .strip_prefix("duplicate field `")
                .and_then(|message| message.split_once('`').map(|(name, _)| name))
                .unwrap_or("config.json");
            ConfigError::field(if name == "allowed_hosts" {
                "http.allowed_hosts"
            } else {
                name
            })
        })
    })?;
    if file
        .drug_approvals_limit
        .is_some_and(|n| !(1..=50).contains(&n))
    {
        return Err(ConfigError::field("drug_approvals_limit"));
    }
    if let Some(keys) = &file.keys {
        for key in keys.keys() {
            if !biomcp_query::sources::keys()
                .iter()
                .any(|name| name.config() == key)
            {
                return Err(ConfigError::field(&format!("keys.{key}")));
            }
        }
    }
    Ok(file)
}
fn diagnose(bytes: &[u8]) -> Result<ConfigFile, ConfigError> {
    let value = serde_json::from_slice(bytes).map_err(|_| ConfigError::field("config.json"))?;
    let mut map = object(value, "config.json")?;
    let http = map
        .remove("http")
        .filter(|v| !v.is_null())
        .map(|value| {
            let mut map = object(value, "http")?;
            let allowed_hosts = take(&mut map, "allowed_hosts")
                .map_err(|_| ConfigError::field("http.allowed_hosts"))?
                .ok_or_else(|| ConfigError::field("http.allowed_hosts"))?;
            unknown(&map, "http.")?;
            Ok(Http { allowed_hosts })
        })
        .transpose()?;
    let keys = map
        .remove("keys")
        .filter(|v| !v.is_null())
        .map(|value| {
            let map = object(value, "keys")?;
            map.into_iter()
                .map(|(key, value)| {
                    let field = format!("keys.{key}");
                    if !biomcp_query::sources::keys()
                        .iter()
                        .any(|name| name.config() == key)
                    {
                        return Err(ConfigError::field(&field));
                    }
                    let value =
                        serde_json::from_value(value).map_err(|_| ConfigError::field(&field))?;
                    Ok((key, value))
                })
                .collect::<Result<BTreeMap<String, String>, ConfigError>>()
        })
        .transpose()?;
    let file = ConfigFile {
        hosted_max_use: take(&mut map, "hosted_max_use")?,
        mcp_tool_mode: take(&mut map, "mcp_tool_mode")?,
        mcp_reply_form: take(&mut map, "mcp_reply_form")?,
        mcp_typed_output_schema: take(&mut map, "mcp_typed_output_schema")?,
        mcp_typed_enum_limit: take(&mut map, "mcp_typed_enum_limit")?,
        mcp_typed_tool_limit: take(&mut map, "mcp_typed_tool_limit")?,
        mcp_typed_definition_bytes: take(&mut map, "mcp_typed_definition_bytes")?,
        format: take(&mut map, "format")?,
        proxy: take(&mut map, "proxy")?,
        http,
        disable_sources: take(&mut map, "disable_sources")?,
        ca_bundle: take(&mut map, "ca_bundle")?,
        ncbi_email: take(&mut map, "ncbi_email")?,
        keys,
        auto_sync_max_bytes: take(&mut map, "auto_sync_max_bytes")?,
        command_timeout_seconds: take(&mut map, "command_timeout_seconds")?,
        drug_approvals_limit: take(&mut map, "drug_approvals_limit")?,
        drug_search_name_limit: take(&mut map, "drug_search_name_limit")?,
        source_pacing: map
            .remove("source_pacing")
            .filter(|v| !v.is_null())
            .map(pacing)
            .transpose()?,
        http_max_calls: take(&mut map, "http_max_calls")?,
        http_max_connections: take(&mut map, "http_max_connections")?,
    };
    unknown(&map, "")?;
    Ok(file)
}
pub(crate) fn read_file(
    override_path: Option<&PathBuf>,
) -> Result<Option<ConfigFile>, ConfigError> {
    config_path(override_path)?
        .map(|path| match fs::read(path) {
            Ok(bytes) => decode(&bytes).map(Some),
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound && override_path.is_none() =>
            {
                Ok(None)
            }
            Err(_) => Err(ConfigError::field("config.json")),
        })
        .transpose()
        .map(Option::flatten)
}

/// Decode one map entry at a time so errors name a source without exposing values.
pub(super) fn pacing(
    value: Value,
) -> Result<BTreeMap<String, biomcp_fetch::PacingOverride>, ConfigError> {
    object(value, "source_pacing")?
        .into_iter()
        .map(|(name, value)| {
            let field = format!("source_pacing.{name}");
            serde_json::from_value(value)
                .map(|value| (name, value))
                .map_err(|_| ConfigError::field(&field))
        })
        .collect()
}
