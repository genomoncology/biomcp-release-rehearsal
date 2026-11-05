# Typed replies carry the JSON as text, under the same cap as single mode

Status: OPEN.
Milestone: 5
Depends on: 0623

## Outcome

Typed mode follows `mcp_reply_form` with default `both`: the first text block starts with the one-line summary and then holds the serialized JSON, and the result size cap counts the JSON value alone in both modes.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` items 1 and 7. Typed mode forces structured-only and one summary line, so clients that read only `content[0].text`, especially on errors, lose the data; and the cap refuses near 70k characters of JSON in single mode but about 150k in typed, skewing the bench.
- Starts from: `tool_reply_form` in `crates/biomcp/src/config.rs`, `summarize` in `crates/biomcp/src/mcp/typed.rs`, and the cap in `crates/biomcp/src/mcp/protocol.rs`.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. Single mode's bytes change only if the cap change moves a refusal, and any such case is named. The design is `planning/typed-mcp-tools.md`.
- Proof: installed typed calls in each reply form, one success and one error, show the summary then the JSON; the same JSON is refused or allowed identically in both modes.
- Changes: `crates/biomcp/src/config.rs`, `crates/biomcp/src/mcp/typed.rs`, `crates/biomcp/src/mcp/protocol.rs`, `crates/biomcp/tests/runner/typed_tools.rs`; reply form, cap and cases.
- Defers: nothing.
