# 0494 — Keep each source page's native window facts in one value

Status: COMPLETE.
Landed: c21c20e21919a33d0289182ec9a45eb1f5c08339

Milestone: 5

Depends on: 0472, 0486

## Outcome

Each adapter that pages a source returns its accepted records together with the native window facts (offset, page size, native total, whether the provider returned nothing) in one named internal value. That value calculates continuation and provider emptiness, and local window selection runs before refused rows are removed. Callers stop rebuilding these facts from the count of accepted records.

- Why: the coordinator's root cause analysis (m15669) found six source corrections in 0472 and 0486 with one cause. Accepted records and native traversal facts travel as separate fields or positional tuples, so callers mistake refused rows for provider emptiness or the end of the results.
- Starts from: `SEERPage` with disease and SEER callers, cBioPortal records and the mutation tuple with study get and variant recurrence, the ChEMBL page and `CellPage` with drug and cell line callers, and `openFDAPage` with label search and the drug query.
- Rules: public `retrieved_count` counts accepted identities, and native received or consumed counts stay internal (m15678); one internal type per adapter family or one shared generic, whichever is smaller; no pagination framework; no public schema change; replies stay byte-identical except where the reply diff names a fixed defect.
- Proof: for each of the four families, a case over recorded replies where refused rows fill a native page fails on a build that derives continuation from the accepted count; the reply diff is otherwise empty.
- Changes: the four adapter families, their callers, cases and baseline.
- Defers: adapters outside these four families, unless the audit finds the same tuple there.
