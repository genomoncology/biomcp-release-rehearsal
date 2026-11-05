# 0656 — fetch returns readable text once

Status: COMPLETE.
Landed: 36b8461491e430c8b052a0517205eae3a49de42a

Milestone: 5

## Outcome

The `fetch` research tool returns the record once, as readable text with no HTML entities such as `&#46;` and no raw `<sup>` tags, and without a second full copy of the same record.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), item 11 and the report's note that `fetch` sends JSON and Markdown together, doubling the payload. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: `crates/biomcp/src/mcp/research.rs` and the Markdown rendering it uses.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Keep the search/fetch contract ChatGPT needs (`decisions/2026-10-23-mcp-is-one-tool-over-the-cli.md`).
- Proof: replayed `fetch article 39214094` asserts no entities or raw tags and one copy; routine gate.
- Changes: `crates/biomcp/src/mcp/research.rs`, `tests/`; the fix and its cases.
- Defers: nothing.
