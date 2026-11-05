# Typed MCP mode joins the 1.0 release as an opt-in setting

Status: OPEN.
Milestone: 5
Depends on: 0629, 0630, 0631, 0632

## Outcome

The reviewed `mcp-typed-tools` branch merges to main, so the 1.0 release candidate offers typed MCP tools behind `mcp_tool_mode = "typed"` (or `serve --tool-mode typed`) with the single tool still the default, and the release candidate is re-cut from that main.

- Why: Ian, 2026-11-05: "we're going to try to get a 1.0 release out to a temporary test server, and I want that test server to be able to test the multi-tool BioMCP tool." Recorded as the 2026-11-05 amendment to `decisions/2026-10-23-mcp-is-one-tool-over-the-cli.md`. This reopens the frozen product source (4250d1952, candidate ed9c36a96) for this one change.
- Starts from: the branch tip after 0624–0632, `planning/typed-mcp-tools.md`, and current main.
- Rules:
  - Single mode is byte-identical to the frozen candidate, with one allowed change: 0628's source-owned `next` step on `get` usage errors for name-like inputs (for example `get variant "BRAF V600E"` now suggests `search variant --query=...` instead of `help get variant`, m25441). Every other field of those errors and every successful reply stay byte-identical; each changed case is a named witness whose new `next` call executes, and the release notes list the change. All other cases, the schema baseline and the single-mode baseline pass unchanged.
  - Typed mode passes 0632's harness check before the merge.
  - Help text changes from 0627's command summaries are allowed and listed in the release notes, with `mcp_tool_mode` and the typed tool list.
  - After the merge, re-cut the candidate (0336's refresh steps, schema baseline export) and run 0293's qualification on the new head: hosted rehearsal on all five platforms, and the local cold matrix since product code changed.
  - Send the hosting team the typed `tools/list` for their wrapper's tool catalog (0630's rule) and the setting name.
- Proof: the combined gate on merged main passes with single-mode bytes unchanged and typed cases green; the new candidate head and its qualification results are recorded in 0293.
- Changes: `crates/biomcp/src/mcp/**`, `crates/biomcp/src/config.rs`, `crates/biomcp/src/config/file.rs`, `crates/biomcp-types/src/query/commands.rs`, `crates/biomcp/src/help.rs`, `crates/biomcp/build/**`, `docs/**`, `release/notes.md`; the merge, release notes and candidate re-cut.
- Defers: making typed the default, until the bench result.
