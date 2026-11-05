# 0663 — Gene search finds a gene by its full name

Status: COMPLETE.
Landed: 41470bbb4632e42b4382fa87fcdd25b0ab558e68

Milestone: 5

## Outcome

`search gene "kirsten rat sarcoma"` finds KRAS: gene search matches full names, previous names and aliases from HGNC, not only the exact symbol or name.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), the gene row of the report: the full name returned 0. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the HGNC gene search in `crates/biomcp-query/src/gene/`.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. HGNC fields only; exact symbol matches still rank first.
- Proof: replayed cases for a full name, an alias and an exact symbol; routine gate.
- Changes: `crates/biomcp-query/src/gene/`, `tests/`; the fix and its cases.
- Defers: nothing.
