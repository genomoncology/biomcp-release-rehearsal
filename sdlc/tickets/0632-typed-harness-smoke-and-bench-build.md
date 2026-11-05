# Typed tools reach the model in real harnesses, then the bench build

Status: OPEN.
Milestone: 5
Depends on: 0624, 0625, 0626, 0627, 0628

## Outcome

Before the bench, one success call and one error call in typed mode are run through pi and Claude Code, and the model quotes a field from each reply; then a Linux x86_64 binary is built from the branch tip and its path, SHA-256 and tool-list size go to the PM for the Biomed Agent Bench.

- Why: `records/2026-11-05-typed-mcp-tools-review.md` closing items: what reaches the model differs by client (some pass only `content[0].text`, especially on errors), and the bench must measure typed tools, not a client quirk. `planning/typed-mcp-tools.md` Bench and merge.
- Starts from: the branch tip after 0624–0631, and the existing MCP smoke cases.
- Rules: Branch only: build and land on `mcp-typed-tools`, never main; the frozen 1.0 candidate is untouched. Use public example IDs only. Record which harness version ran and what the model saw; a harness that hides the data is reported, not worked around. 0629, 0630 and 0631 should land first if ready, but do not block the build.
- Proof: a recorded transcript per harness showing the quoted fields, and the binary's path and SHA-256 from a `pm job`. If Ian has not confirmed pi's spend scope when the rest is done, 0632 completes with the Claude Code transcript and records pi as not run; the pi run then happens before the bench, not before the 0636 merge (m25603).
- Changes: `crates/biomcp/tests/runner/typed_tools.rs`, `scripts/typed_tools_check.sh`; the smoke record and the bench build.
- Defers: Amazon Quick, until the hosting team can reach it.
