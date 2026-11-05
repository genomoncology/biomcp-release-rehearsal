# Typed gates fail when they prove nothing

Status: COMPLETE. Reviewed typed implementation landed through 0636 at 95e7be997f3bc038111fd65527e03ce4db1e5d2a; completion requested by the PM in m26076. Earlier branch-only ordering was superseded by the approved main integration.
Milestone: 5
Depends on: 0623

## Outcome

The single-mode byte check fails when its baseline is missing, and typed replay calls every typed tool at least once, runs every recorded case for each tool rather than stopping at the first pass, and names any case the typed schema rejects instead of skipping it.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` item 10. The byte check passes when `BIOMCP_TYPED_BASELINE` is unset (`typed_baseline.rs:11-13`); replay covers 35 of 58 tools, stops at the first passing case, and skips schema-rejected cases, so it cannot catch a schema that refuses valid CLI input.
- Starts from: `crates/biomcp/tests/runner/typed_baseline.rs`, `crates/biomcp/tests/runner/typed_replay.rs`, and `scripts/typed_tools_check.sh`.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. No new framework; extend the existing replay. A tool with no recorded case is named in the report and fails unless listed with a reason.
- Proof: the byte check fails with no baseline; replay reports a call for every tool; a synthetic over-strict schema makes replay fail.
- Changes: `crates/biomcp/tests/runner/typed_baseline.rs`, `crates/biomcp/tests/runner/typed_replay.rs`, `scripts/typed_tools_check.sh`; the gate changes.
- Defers: nothing.
