//! The process argument and settings boundary.
pub(crate) mod credentials;
pub(crate) mod key_file;
mod pacing;
use crate::grammar::Globals;
use biomcp_fetch::{Fetch, FetchConfig, Mode, ReplayCache, Secret, SecretName, Secrets};
use std::{ffi::OsString, fs, path::PathBuf, sync::Arc, time::Duration};
#[cfg(test)]
mod credential_reads_tests;

pub(crate) mod file;
pub(crate) use file::ConfigError;
use file::ConfigFile;

pub(crate) const DEFAULT_HTTP_CAPACITY: usize = 8;

pub(crate) fn http_capacity(
    globals: &Globals,
    settings: &Settings,
) -> Result<(usize, usize), ConfigError> {
    let file = configured(settings)?;
    let calls = globals
        .http_max_calls
        .or_else(|| file.and_then(|f| f.http_max_calls))
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(DEFAULT_HTTP_CAPACITY);
    let connections = globals
        .http_max_connections
        .or_else(|| file.and_then(|f| f.http_max_connections))
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(DEFAULT_HTTP_CAPACITY);
    if connections < calls || connections > (usize::MAX - 1) / 2 {
        return Err(ConfigError::field("http_max_connections"));
    }
    Ok((calls, connections))
}

pub(crate) type Settings = Result<Option<ConfigFile>, ConfigError>;

fn configured(settings: &Settings) -> Result<Option<&ConfigFile>, ConfigError> {
    settings.as_ref().map(Option::as_ref).map_err(Clone::clone)
}
pub(crate) fn http_hosts(settings: &Settings) -> Result<Vec<String>, ConfigError> {
    Ok(configured(settings)?
        .and_then(|file| file.http.as_ref())
        .map(|http| http.allowed_hosts.clone())
        .unwrap_or_default())
}
#[expect(
    clippy::disallowed_methods,
    reason = "Arguments enter only at the command boundary"
)]
pub(crate) fn arguments() -> Vec<OsString> {
    std::env::args_os().skip(1).collect()
}
#[expect(
    clippy::disallowed_methods,
    reason = "Only the settings boundary reads the process environment"
)]
fn environment(name: &str) -> Result<Option<String>, ConfigError> {
    match std::env::var(name) {
        Ok(value) => Ok((!value.is_empty()).then_some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(ConfigError::field(name)),
    }
}

fn variable(name: &str) -> Result<Option<PathBuf>, ConfigError> {
    environment(name).map(|value| value.map(PathBuf::from))
}

pub(crate) fn retired_study_directory() -> bool {
    match variable("BIOMCP_STUDY_DIR") {
        Ok(value) => value.is_some(),
        Err(_) => true,
    }
}

pub(crate) const BUDGET: Duration = Duration::from_secs(20);
#[derive(Clone, Debug)]
pub(crate) enum StartupError {
    Cache(biomcp_fetch::CacheError),
    Configuration(ConfigError),
    Local,
}
impl StartupError {
    pub(crate) fn is_pacing(&self) -> bool {
        matches!(self, Self::Configuration(error) if error.is_pacing())
    }
}
impl From<ConfigError> for StartupError {
    fn from(error: ConfigError) -> Self {
        Self::Configuration(error)
    }
}
impl From<biomcp_fetch::CacheError> for StartupError {
    fn from(error: biomcp_fetch::CacheError) -> Self {
        Self::Cache(error)
    }
}
pub(crate) fn fetch(globals: &Globals, settings: &Settings) -> Result<Fetch, StartupError> {
    let home = variable("HOME")?;
    let file = configured(settings)?;
    fetch_using(globals, home, file)
}
pub(crate) fn ncbi_email(settings: &Settings) -> Result<Option<Secret>, ConfigError> {
    Ok(configured(settings)?
        .and_then(|file| file.ncbi_email.clone())
        .filter(|s| !s.is_empty())
        .map(Secret::new))
}
fn fetch_using(
    globals: &Globals,
    home: Option<PathBuf>,
    file: Option<&ConfigFile>,
) -> Result<Fetch, StartupError> {
    let cache = cache_path_using(home)?;
    let mode = if let Some(dir) = &globals.replay {
        Mode::Replay { dir: dir.clone() }
    } else if let Some(dir) = &globals.record {
        Mode::Record { dir: dir.clone() }
    } else if globals.offline {
        Mode::Offline
    } else {
        Mode::Live {
            fresh: globals.fresh,
        }
    };
    let replay_cache = globals
        .case_cache
        .as_ref()
        .map(|path| {
            if globals.replay.is_some() && !path.exists() {
                ReplayCache::create_case_with_owners(path, &[biomcp_query::store::OWNER])
            } else {
                ReplayCache::open_case_with_owners(path, &[biomcp_query::store::OWNER])
            }
        })
        .transpose()
        .map_err(StartupError::from)?;
    if !matches!(mode, Mode::Replay { .. })
        && replay_cache.is_none()
        && let Some(parent) = cache.as_ref().and_then(|path| path.parent())
    {
        create_cache_directory(parent).map_err(|_| StartupError::Local)?;
    }
    let secrets = secrets_from(file);
    let network = network(file)?;
    Fetch::open(FetchConfig {
        rate_share: globals.rate_share,
        source_pacing: pacing::resolve(globals, file)?,
        disabled_sources: disabled_sources(globals, file)?,
        disabled_uses: biomcp_query::sources::disabled_uses(&disabled_names(globals, file)),
        observations: globals
            .observe
            .as_ref()
            .map(|path| biomcp_fetch::Observations::create(path).map(std::sync::Arc::new))
            .transpose()
            .map_err(|_| StartupError::Local)?,
        cache_owners: vec![biomcp_query::store::OWNER],
        secrets,
        network,
        cache_predecessors: crate::paths::cache_predecessors(cache.as_deref()),
        cache,
        mode,
        replay_cache,
        auto_sync_max_bytes: file
            .as_ref()
            .and_then(|file| file.auto_sync_max_bytes)
            .unwrap_or_else(|| FetchConfig::default().auto_sync_max_bytes),
        proxy: file.and_then(|file| file.proxy.clone()).map(Secret::new),
        ..FetchConfig::default()
    })
    .map_err(|error| match error {
        biomcp_fetch::FetchError::Cache(error) => error.into(),
        biomcp_fetch::FetchError::Proxy => ConfigError::field("proxy").into(),
        _ => StartupError::Local,
    })
}

fn create_cache_directory(path: &std::path::Path) -> std::io::Result<()> {
    let mut directory = fs::DirBuilder::new();
    directory.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(path)
}

fn disabled_sources(
    globals: &Globals,
    file: Option<&ConfigFile>,
) -> Result<std::collections::BTreeSet<biomcp_types::Source>, ConfigError> {
    biomcp_query::sources::disabled(&disabled_names(globals, file))
        .map_err(|_| ConfigError::field("disable_sources"))
}
fn disabled_names(globals: &Globals, file: Option<&ConfigFile>) -> Vec<String> {
    let mut names = globals.disable_sources.clone();
    names.extend(
        file.and_then(|f| f.disable_sources.clone())
            .unwrap_or_default(),
    );
    names
}

fn network(file: Option<&ConfigFile>) -> Result<biomcp_fetch::Network, ConfigError> {
    use std::io::Read;
    const MAX_PEM_BYTES: u64 = 4 * 1024 * 1024;
    let roots = file
        .and_then(|file| file.ca_bundle.as_ref())
        .map(|path| {
            let mut bytes = Vec::new();
            fs::File::open(path)
                .map_err(|_| ConfigError::field("ca_bundle"))?
                .take(MAX_PEM_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| ConfigError::field("ca_bundle"))?;
            if bytes.len() as u64 > MAX_PEM_BYTES {
                return Err(ConfigError::field("ca_bundle"));
            }
            biomcp_fetch::Certificates::from_pem(&bytes)
                .map_err(|_| ConfigError::field("ca_bundle"))
        })
        .transpose()?
        .unwrap_or_default();
    let explicit = file.is_some_and(|file| file.proxy.is_some());
    let setting = |lower, upper| {
        if explicit {
            Ok((upper, None))
        } else {
            network_variable(lower, upper)
        }
    };
    let (http_name, http) = setting("http_proxy", "HTTP_PROXY")?;
    let (https_name, https) = setting("https_proxy", "HTTPS_PROXY")?;
    let (bypass_name, bypass) = setting("no_proxy", "NO_PROXY")?;
    let proxy = |value: Option<String>| value.filter(|value| !value.is_empty()).map(Secret::new);
    biomcp_fetch::Network::new(
        proxy(http),
        proxy(https),
        &bypass.unwrap_or_default(),
        roots,
    )
    .map_err(|error| {
        ConfigError::field(match error {
            biomcp_fetch::FetchError::HttpProxy => http_name,
            biomcp_fetch::FetchError::HttpsProxy => https_name,
            _ => bypass_name,
        })
    })
}

#[expect(
    clippy::disallowed_methods,
    reason = "Only the settings boundary reads the process environment"
)]
fn network_variable(
    lower: &'static str,
    upper: &'static str,
) -> Result<(&'static str, Option<String>), ConfigError> {
    // Presence, including an empty lower-case value, takes precedence on Unix.
    if !cfg!(windows) {
        match std::env::var(lower) {
            Ok(value) => return Ok((lower, Some(value))),
            Err(std::env::VarError::NotPresent) => {}
            Err(std::env::VarError::NotUnicode(_)) => return Err(ConfigError::field(lower)),
        }
    }
    match std::env::var(upper) {
        Ok(value) => Ok((upper, Some(value))),
        Err(std::env::VarError::NotPresent) => Ok((upper, None)),
        Err(std::env::VarError::NotUnicode(_)) => Err(ConfigError::field(upper)),
    }
}

pub(crate) fn format(settings: &Settings) -> Option<crate::grammar::Format> {
    configured(settings)
        .ok()
        .flatten()
        .and_then(|file| file.format)
}

pub(crate) fn config_path(override_path: Option<&PathBuf>) -> Result<Option<PathBuf>, ConfigError> {
    if let Some(path) = override_path {
        return Ok(Some(path.clone()));
    }
    Ok(crate::paths::config(
        variable("XDG_CONFIG_HOME")?,
        variable("HOME")?,
        variable("APPDATA")?,
    ))
}
pub(crate) fn secrets(settings: &Settings) -> Result<Secrets, ConfigError> {
    Ok(secrets_from(configured(settings)?))
}
pub(crate) fn has_stored_key(settings: &Settings, name: SecretName) -> Result<bool, ConfigError> {
    Ok(configured(settings)?
        .and_then(|file| file.keys.as_ref())
        .and_then(|keys| keys.get(name.config()))
        .is_some_and(|key| !key.is_empty()))
}
fn secrets_from(file: Option<&ConfigFile>) -> Secrets {
    secrets_using(file.and_then(|file| file.keys.clone()), variable)
}
fn secrets_using(
    keys: Option<std::collections::BTreeMap<String, String>>,
    environment: impl Fn(&str) -> Result<Option<PathBuf>, ConfigError> + Send + Sync + 'static,
) -> Secrets {
    let mut secrets = Secrets::default();
    let keys = Arc::new(keys);
    let environment = Arc::new(environment);
    for name in biomcp_query::sources::keys() {
        let keys = keys.clone();
        let environment = environment.clone();
        secrets.defer(name, move || {
            configured_secret(keys.as_ref().as_ref(), name, environment.as_ref())
        });
    }
    secrets
}
fn configured_secret(
    keys: Option<&std::collections::BTreeMap<String, String>>,
    name: SecretName,
    environment: &impl Fn(&str) -> Result<Option<PathBuf>, ConfigError>,
) -> Result<Option<Secret>, biomcp_fetch::InputError> {
    if let Some(value) = keys
        .and_then(|keys| keys.get(name.config()))
        .filter(|v| !v.is_empty())
    {
        return Ok(Some(Secret::new(value.clone())));
    }
    environment(name.environment())
        .map_err(|_| biomcp_fetch::InputError::CredentialEncoding)?
        .map(|value| {
            value
                .into_os_string()
                .into_string()
                .map(Secret::new)
                .map_err(|_| biomcp_fetch::InputError::CredentialEncoding)
        })
        .transpose()
}

pub(crate) fn has_environment_key(name: SecretName) -> Result<bool, ConfigError> {
    Ok(variable(name.environment())?.is_some())
}

/// Apply per-call policy as a union with the already configured transport policy.
pub(crate) fn restrict(fetch: Fetch, globals: &Globals) -> std::io::Result<Fetch> {
    let disabled = biomcp_query::sources::disabled(&globals.disable_sources)
        .map_err(|_| std::io::Error::other("unchecked source policy"))?;
    Ok(fetch
        .restrict_sources(disabled)
        .restrict_uses(biomcp_query::sources::disabled_uses(
            &globals.disable_sources,
        )))
}

pub(crate) fn cache_path() -> Result<Option<PathBuf>, ConfigError> {
    cache_path_using(variable("HOME")?)
}
fn cache_path_using(home: Option<PathBuf>) -> Result<Option<PathBuf>, ConfigError> {
    Ok(crate::paths::cache(
        variable("XDG_CACHE_HOME")?,
        home,
        variable("LOCALAPPDATA")?,
    ))
}
#[expect(
    clippy::disallowed_methods,
    reason = "The settings boundary resolves the running executable"
)]
pub(crate) fn executable() -> std::io::Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let Some(path) =
        variable("PATH").map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))?
    else {
        return Ok(executable);
    };
    let identity = executable.canonicalize()?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(format!("biomcp{}", std::env::consts::EXE_SUFFIX));
        if candidate.canonicalize().is_ok_and(|path| path == identity) {
            return std::path::absolute(candidate);
        }
    }
    Ok(executable)
}
