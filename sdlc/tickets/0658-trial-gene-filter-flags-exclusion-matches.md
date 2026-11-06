# 0658 — Trial gene filter flags trials that only exclude the gene

Status: COMPLETE.
Landed: f36bbacbf25b48274c1f54a96acd9f4f857790e2

Milestone: 5

## Outcome

`search trial --gene KRAS` marks each trial whose only mention of the gene is in its exclusion criteria, or that has no gene link beyond a word match, the same way condition mismatches are marked today.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), item 14: NCT06998940 excludes KRAS mutations and a pNET trial had no KRAS link, but both matched without a warning. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the trial search and its condition-mismatch notice in `crates/biomcp-query/src/trial/`.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. BioMCP reports where the gene appears; it never drops a trial or judges eligibility.
- Proof: replayed cases for an exclusion-only trial and an inclusion trial; routine gate.
- Changes: `crates/biomcp-query/src/trial/`, `tests/`; the fix and its cases.
- Defers: nothing.
