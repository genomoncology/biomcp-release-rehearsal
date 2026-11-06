//! Navigation, command options and shared process options have separate help pages.
use std::path::Path;

pub(super) fn check(root: &Path) -> super::Check {
    // Entity navigation deliberately omits arguments in both JSON and Markdown.
    // Keep the original command and global-flag assertions on their owning pages.
    super::put(
        root,
        "markdown-pair.yaml",
        concat!(
            "  - run: [help, author, --format, json]\n",
            "  - run: [help, author, --format, md]\n",
            "    max_bytes: 8192\n",
            "    expect: {'': {includes_all_of: ['biomcp author papers']}}\n",
            "  - run: [help, author, papers, --format, json]\n",
            "  - run: [help, author, papers, --format, md]\n",
            "    max_bytes: 8192\n",
            "    expect: {'': {includes_all_of: ['biomcp author papers', '--limit', '--offset', '--pool', '--full']}}\n",
            "  - run: [help, help, --format, json]\n",
            "  - run: [help, help, --format, md]\n",
            "    max_bytes: 8192\n",
            "    expect: {'': {includes_all_of: ['--format', '--disable-sources']}}\n",
        ),
    )?;
    let accepted = super::invoke(root, &["markdown-pair.yaml", "--exact"])?;
    if !accepted.status.success() {
        return Err(format!(
            "matching Markdown pair failed: {}; stderr {}",
            String::from_utf8_lossy(&accepted.stdout),
            String::from_utf8_lossy(&accepted.stderr),
        ));
    }
    Ok(())
}
