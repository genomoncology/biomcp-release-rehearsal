# 0662 — FAERS returns reaction counts

Status: COMPLETE.
Landed: 28f9abe29662753314a3a68f8d52526f370fc311

Milestone: 5

## Outcome

`search adverse-event` for a drug can return reaction counts for its reports, from openFDA's `count` endpoint, so a safety summary takes one call instead of one per report.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), capability 6: rows held only ID, date and a serious flag. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the FAERS adapter in `crates/biomcp-query/src/adverse_event/`.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Counts are openFDA's own; BioMCP adds no rates or interpretation and keeps FAERS's reporting caveats.
- Proof: replayed count case for one drug; routine gate.
- Changes: `crates/biomcp-query/src/adverse_event/`, `tests/`; the fix and its cases.
- Defers: nothing.

## Implementation evidence

Both single and typed baseline calls returned report headers and rejected aggregation (j30640). `--aggregate reactions` now requests openFDA's exact reaction-term count endpoint once and exposes native `term`/`count` buckets separately from report rows. It adds no report total, rate or interpretation. FAERS caveats and MedDRA admission remain in force; empty results omit the optional bucket collection.

The outside-in case failed before implementation (j30650). Focused proof j30753 covers CLI/single-MCP parity, typed named calls, large counts, absence, disabled MedDRA, invalid options, malformed replies and existing report searches. The shared typed reply helper comes from the separately proven research-fetch fix (j30701).

Lesson: run Clippy across all affected crates with `--keep-going` before the routine gate, so an early compilation blocker does not hide the next diagnostic.
