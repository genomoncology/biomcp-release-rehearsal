use super::Check;
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use serde_json::Value;
use std::path::Path;

pub(super) fn replay(
    capture: &str,
    mut change: impl FnMut(&str, &mut Value),
) -> Check<(tempfile::TempDir, Fetch)> {
    let root = tempfile::tempdir()?;
    let target = root.path().join("recordings");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/replies")
        .join(capture);
    super::recordings::copy_capture(&source, &target, |source, record| {
        change(source, record);
        super::recordings::cache_acquisition(record, 42);
    })?;
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: target },
        cache: Some(root.path().join("cache.sqlite")),
        ..Default::default()
    })?;
    Ok((root, fetch))
}
