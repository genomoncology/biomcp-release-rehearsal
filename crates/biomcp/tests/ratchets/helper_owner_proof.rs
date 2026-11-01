//! Copied mapping and protocol helpers fail the source ownership contract.
use super::{Check, rejects};

pub(super) fn check() -> Check {
    for declaration in [
        "fn bad() {}",
        "fn bad_reply() {}",
        "fn received_date() {}",
        "fn root() {}",
    ] {
        rejects(
            "crates/biomcp-adapters/src/example/map.rs",
            declaration,
            "copied helper",
        )?;
    }

    for declaration in ["fn problem() {}", "fn text() {}"] {
        rejects(
            "crates/biomcp-adapters/src/example/map.rs",
            declaration,
            "mapping helpers belong in mapped.rs",
        )?;
    }
    shared_test_owner()
}

fn shared_test_owner() -> Check {
    let tree = super::tree()?;
    let root = tree.path();
    let manifest = "crates/biomcp-test-support/Cargo.toml";
    let source = "crates/biomcp-test-support/src/lib.rs";
    let fixture = "//! Shared test fixtures.\n#[expect(clippy::disallowed_methods, reason=\"Fixture I/O owner\")]\nfn fixture() {}\n";
    super::put(
        root,
        manifest,
        "[package]\nname='biomcp-test-support'\npublish=false\n",
    )?;
    super::put(root, source, fixture)?;
    super::check(root)?;
    super::put(
        root,
        manifest,
        "[package]\nname='biomcp-test-support'\npublish=true\n",
    )?;
    assert!(
        super::check(root).is_err(),
        "publishable support crate passed"
    );
    super::put(
        root,
        manifest,
        "[package]\nname='biomcp-test-support'\npublish=false\n",
    )?;
    for content in [format!("//! {}", "x".repeat(161)), "\n".repeat(601)] {
        super::put(root, source, &content)?;
        assert!(
            super::check(root).is_err(),
            "shared fixture size or width limit passed"
        );
    }
    super::put(root, source, fixture)?;
    super::put(
        root,
        "crates/other-private/Cargo.toml",
        "[package]\npublish=false\n",
    )?;
    super::put(root, "crates/other-private/src/lib.rs", fixture)?;
    assert!(
        super::check(root).is_err(),
        "unrelated private crate gained fixture exemption"
    );
    Ok(())
}
