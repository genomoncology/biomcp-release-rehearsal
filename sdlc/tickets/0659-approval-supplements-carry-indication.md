# 0659 — Approval supplements say what they approved

Status: OPEN.

Milestone: 5

## Outcome

Each Drugs@FDA efficacy supplement row in `get drug <app> approvals --source drugsfda` carries the indication text it added when a public FDA source states it, and says plainly when none does. The approval rows come back newest first, so the latest action is the first row.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), capability 2: supplements showed only `EFFICACY` and a date, so the agent read the letter or guessed; Cabometyx's 15 approval rows also arrived in mixed date order. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the Drugs@FDA approvals section in `crates/biomcp-query/src/drug/`, and the openFDA label versions by application number.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Text comes only from FDA's own records (label version or letter); BioMCP never infers it. If no public source states it for a supplement, the row says so.
- Proof: replayed cases for a supplement with stated indication text and one without, and for an application whose source rows arrive out of date order; routine gate.
- Changes: `crates/biomcp-query/src/drug/`, `tests/`; the fix and its cases.
- Defers: nothing.
