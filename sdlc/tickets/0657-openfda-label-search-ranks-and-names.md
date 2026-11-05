# 0657 — openFDA label search ranks its hits and names every one

Status: COMPLETE.
Landed: 290b2f278a7c411eb72e791df2bb4f2ffc996e90

Milestone: 5

## Outcome

`search drug pancreatic --source openfda-label` puts labels whose name or indication matches before incidental text matches, every row has a display name, and the reply's total matches the rows it can page through (the hosted run showed total 1 with no rows for `search drug melanoma --source openfda-label`).

- Why: the hosted evaluation of 2026-11-05 (hosting m26488; evidence under `data/biomcp-hosting-evidence/2026-11-05-claude-eval/`), items 12 and 13 and hosting m26471: 8,955 unranked hits led by carbamazepine and gabapentin, nameless rows, and total 1 with zero rows. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).
- Starts from: the openFDA label search in `crates/biomcp-query/src/drug/`, `docs/cautions.md` on its search order, and 0605's label search page.
- Rules: Investigate first: reproduce on current main in single and typed mode with the evaluation's command; if main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Ranking uses openFDA's own field order (name fields, then indications, then other text), no hand-tuned weights. 0651 adds the explicit indication filter; this ticket fixes free-text order and names.
- Proof: replayed cases for a disease word, a nameless label and the total/rows mismatch; routine gate.
- Changes: `crates/biomcp-query/src/drug/`, `tests/`, `docs/cautions.md`; the fix and its cases.
- Defers: nothing.
