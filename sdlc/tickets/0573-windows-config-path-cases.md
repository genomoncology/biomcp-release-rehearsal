# Windows configuration path cases

Status: COMPLETE.
Landed: c0ad78b47519c5f7db46e2991768b2ab3966b122
Milestone: 5

## Outcome

On installed Windows, `housekeeping/config-path-fallbacks` and `housekeeping/offline-provenance-and-config` pass: `mcp-config` names the running executable when PATH is missing or lacks it, and `version --verbose` shows the same config path the settings use.

- Why: private rehearsal run 41376870296 at 0fea901b failed exactly these two cases on Windows, 521 passed (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41376870296/windows-job.log` lines 1583-1594). The messages come from `crates/biomcp/tests/runner/housekeeping_path.rs:51` and `crates/biomcp/tests/runner/housekeeping.rs:74`. Both compare a path the product prints with a path the proof builds. One hypothesis, not yet shown, is a shared Windows path form: the proof's `canonicalize()` returns a `\\?\` verbatim path, or separators or drive-letter case differ.
- Starts from: the two proof files, the product's executable and config path resolution for `mcp-config` and `version --verbose`, and the raw log.
- Rules: bounded diagnosis first: capture the two paths each case compared on Windows. If the product prints a path a Windows user's MCP client cannot use, or one that differs from the config file it actually reads, fix the product. If only the proof compares the wrong form, fix the proof to compare the same file, not looser text. Split into two tickets if the causes differ.
- Proof: a test that fails for the Windows path form the diagnosis names, and the next private rehearsal on Windows.
- Changes: the product's path display or the two proofs, whichever the diagnosis names.
- Defers: other rehearsal failures, each to its own ticket.
