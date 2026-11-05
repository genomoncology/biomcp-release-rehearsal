# Typed mode keeps help, search and fetch, with a typed front map

Status: OPEN.
Milestone: 5
Depends on: 0624

## Outcome

Typed `tools/list` also offers `biomcp_help` (a command path in, help and schema out) and the existing `search` and `fetch` research tools, and typed instructions start with one useful line and a front map that names tools rather than CLI syntax.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` items 8 and 10. Typed mode drops `search` and `fetch`, which the 2026-10-25 amendment to `decisions/2026-10-23-mcp-is-one-tool-over-the-cli.md` requires for ChatGPT, has no help tool, and its instructions say "help sources" in CLI syntax.
- Starts from: `crates/biomcp/src/mcp/typed.rs` `tools`, `crates/biomcp/src/mcp/research.rs`, `instructions` in `crates/biomcp/src/mcp/protocol.rs`, and `crates/biomcp/build/front_map.rs`.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. The typed map is generated from the same registry as the CLI map, within 4,096 characters, most useful line first.
- Proof: typed `tools/list` lists the three tools; `biomcp_help` for one path returns its help; `search` and `fetch` match single mode's; the instructions fit and name tools.
- Changes: `crates/biomcp/src/mcp/typed.rs`, `crates/biomcp/src/mcp/research.rs`, `crates/biomcp/src/mcp/protocol.rs`, `crates/biomcp/build/front_map.rs`, `crates/biomcp/tests/runner/typed_tools.rs`; the tools, map and cases.
- Defers: nothing.
