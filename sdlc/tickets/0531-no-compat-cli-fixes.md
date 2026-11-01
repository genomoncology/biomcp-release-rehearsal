# 0531 — Drop compatibility options and fix small CLI defects

Status: COMPLETE.
Landed: dfc6d1a9b88b8635d4122b79d39825f23a20ddc1

Milestone: 5

## Outcome

The CLI keeps no option for old callers, and five small defects are fixed: trial pages report their real offset, `key set` reads input before it locks the config, MCP saves refuse any path component that starts with a dot, `key list` has its own help text, and `ErrorStatus` describes itself correctly.

- Why: release review C6, A7, A9, B5 and two leftovers of the 2026-10-27 product findings, weighed 2026-11-01, under Ian's stance in `decisions/2026-11-01-no-backwards-compatibility.md`. `crates/biomcp/src/grammar/arguments.rs:75-93` keeps `-q/--query` beside positional text on cell-line and author search; `grammar/local.rs:120-123` keeps a no-op `mcp-config --absolute-path`. `crates/biomcp-query/src/trial.rs:132` sets `offset: 0` on every page. `keys.rs:134` takes the lock and line 141 reads input, so a second `key set` hangs. `save.rs:30-57` with `command/mcp_policy.rs:36-49` lets an MCP save create a missing dotfile such as `~/.ssh/authorized_keys`. `grammar/metadata.rs:18` gives `key list` the wrong description, and `biomcp-types/src/response/error.rs:29` calls `ErrorStatus` "The status of an invalid schema request."
- Starts from: those files, the author and cell-line slice rulings that kept the options, and the upgrade table.
- Rules: remove the two options and their cases, and add each to the upgrade table. The `semanticscholar:` author ID alias lives in `crates/biodata`, which is frozen until 0459 lands, so BioData removes it in its crate and 0459 takes it. Carry the trial offset in the next-page query. Read the secret first, then lock, reread and recheck the replace rule. The dot refusal applies in MCP mode only. Regenerate schemas with `make schema-generate`.
- Proof: cases show each removed option refused with `usage.arguments`, page two of a trial search reporting the offset of its first row, an MCP save to `.zshenv` refused, and the corrected help and schema text. A trial shows a second `key set` does not block while the first waits for input.
- Changes: the grammar, trial paging, keys, the MCP save policy, help metadata, the error type, generated schemas, the upgrade table and the cases.
- Defers: nothing.
