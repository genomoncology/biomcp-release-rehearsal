# Drug approvals list newest first

Status: COMPLETE.
Landed: afcafd4d118ecb71ea38b81010f2a55fe2eac6e6
Milestone: 5

## Outcome

The Drugs@FDA approvals section lists submissions newest first by status date, so the default page of 10 holds the most recent approvals.

- Why: `planning/eval-recommendations-2026-11-04.md` item 2 lists "drug approvals newest first, capped at 10 rows" in the bundle that beat the cold agent. Main caps at 10 (`DEFAULT_DRUG_APPROVALS_LIMIT`) but `submissions()` in `crates/biomcp-adapters/src/drugsfda/map.rs` returns native order, so an old drug's first page can be decades-old supplements.
- Starts from: `submissions()` in `map.rs` and `approvals.rs` in `crates/biomcp-query/src/drug/drugsfda/`.
- Rules: sort by submission status date descending, then submission number descending; a submission with no date sorts last. Paging and totals keep working on the sorted order. The reply says the order.
- Proof: an installed-binary case on a drug with more than 10 submissions shows the newest first, and page two continues without repeats or gaps.
- Changes: `crates/biomcp-adapters/src/drugsfda/**`, `crates/biomcp-query/src/drug/drugsfda/**`, `tests/cases/drug/drugsfda/**`, `tests/replies/drugsfda/**`; the adapter sort, the section text, and cases.
- Defers: nothing.
