//! Isolated replay source setup shared by adapter and query contracts.
use super::capture::copy_capture;
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use serde_json::Value;
type Check<T> = Result<T, Box<dyn std::error::Error>>;
/// Open an isolated replay fetcher from a named public capture.
pub fn replay(
    capture: &str,
    change: impl FnMut(&str, &mut Value),
) -> Check<(tempfile::TempDir, Fetch)> {
    let root = tempfile::tempdir()?;
    let recordings = root.path().join("recordings");
    copy_capture(capture, &recordings, change)?;
    let mut secrets = biomcp_fetch::Secrets::default();
    for name in [
        biomcp_fetch::SecretName::OncokbApiKey,
        biomcp_fetch::SecretName::NciApiKey,
    ] {
        secrets.insert(name, biomcp_fetch::Secret::new("synthetic-replay-key"));
    }
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: recordings },
        cache: Some(root.path().join("cache.sqlite")),
        secrets,
        ..Default::default()
    })?;
    Ok((root, fetch))
}
