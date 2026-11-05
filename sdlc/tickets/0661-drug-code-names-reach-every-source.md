# 0661 — Drug code names and synonyms reach every drug source

Status: COMPLETE.
Landed: 7f6a35390ee8f1e4db200f7485b6a2c1521b0db0

Milestone: 5

## Outcome

A drug code name or synonym such as RMC-6236 or daraxonrasib finds the drug in every source that has it under any of its names, and a source that has none says which other source found it.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), capability 4: daraxonrasib and RMC-6236 returned 0 in ChEMBL and Open Targets with no hint. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: drug name resolution in `crates/biomcp-query/src/drug/` and 0599's name handling.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Synonyms come from the drug sources BioMCP already reads (ChEMBL synonyms, MyChem, openFDA names); a new name service such as RxNorm needs its own ticket and license check.
- Proof: replayed cases for a code name and a synonym across two sources; routine gate.
- Changes: `crates/biomcp-query/src/drug/`, `tests/`; the fix and its cases.
- Defers: nothing.
