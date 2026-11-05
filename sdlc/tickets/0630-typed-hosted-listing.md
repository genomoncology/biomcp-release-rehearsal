# Typed hosted listing shows only what the server will serve

Status: OPEN.
Milestone: 5
Depends on: 0629

## Outcome

In hosted typed mode, `tools/list` omits tools every hosted call would refuse, source enums and descriptions name only sources the serving mode allows, and `readOnlyHint` is set per tool, true for every tool that cannot write a file.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` item 9. Hosted typed mode lists `article_pdf` and `trial_document`, which always refuse there; restricted mode advertises refused sources; stdio marks all 58 tools as not read-only though 4 can write, so confirming clients prompt on every call.
- Starts from: `crates/biomcp/src/mcp/typed.rs` `tools`, `crates/biomcp/src/command/hosted_use.rs`, and the save flags in the command schemas.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. The listing is computed from the same hosted policy that refuses calls, never a separate list. The hosting team refreshes its wrapper's tool catalog from this listing before offering typed mode; send them the listing when it lands.
- Proof: hosted typed `tools/list` under `restricted` omits the save-only tools and refused sources; a call to an omitted tool still refuses as today; per-tool `readOnlyHint` matches whether the command can write.
- Changes: `crates/biomcp/src/mcp/typed.rs`, `crates/biomcp/src/mcp/typed/arguments.rs`, `crates/biomcp/src/command/hosted_use.rs`, `crates/biomcp/tests/runner/typed_tools.rs`; the listing filter, hints and cases.
- Defers: nothing.
