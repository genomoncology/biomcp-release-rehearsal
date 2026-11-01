//! Shared public fixture root lookup.
use std::path::Path;
#[expect(
    clippy::disallowed_methods,
    reason = "Installed checks copy public fixtures outside the checkout"
)]
/// Locate the public test fixtures, honoring the installed-fixture override.
pub fn fixtures() -> std::path::PathBuf {
    std::env::var_os("BIOMCP_TEST_FIXTURES").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests"),
        std::path::PathBuf::from,
    )
}
