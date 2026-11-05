# 0662 — FAERS returns reaction counts

Status: OPEN.

Milestone: 5

## Outcome

`search adverse-event` for a drug can return reaction counts for its reports, from openFDA's `count` endpoint, so a safety summary takes one call instead of one per report.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), capability 6: rows held only ID, date and a serious flag. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the FAERS adapter in `crates/biomcp-query/src/adverse_event/`.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Counts are openFDA's own; BioMCP adds no rates or interpretation and keeps FAERS's reporting caveats.
- Proof: replayed count case for one drug; routine gate.
- Changes: `crates/biomcp-query/src/adverse_event/`, `tests/`; the fix and its cases.
- Defers: nothing.
