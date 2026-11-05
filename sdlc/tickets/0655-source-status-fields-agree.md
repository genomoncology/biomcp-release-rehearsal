# 0655 — Source status fields never contradict each other

Status: COMPLETE.
Landed: 31b7b283063a00afb82aaa6c025091df4dc59b40

Milestone: 5

## Outcome

A source row's `state` and `reason` always agree: a source that did not answer for an unsupported filter is not `ok`, and a join that found no ID is not `data`.

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), item 10: fda-cdx returned `state: ok` with `reason: unsupported`; hotspots returned `state: data` with `reason: no-join-id`. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the source report types in `crates/biomcp-types/src/response/` and the adapters that set them.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Fix the type so the contradiction cannot be built, rather than a check after the fact; keep the public state and reason vocabularies unless a schema change is unavoidable, and then record it in the schema review.
- Proof: type-level tests and the two replayed cases; schema comparison; routine gate.
- Changes: `crates/biomcp-types/src/response/`, `crates/biomcp-query/src/`, `tests/`; the fix and its cases.
- Defers: nothing.
