# 0461 — Turn real user queries and known problem shapes into cases

Status: COMPLETE.
Landed: 7bf54e4ebbb1902ad482938f000d0dfe0990ee7d

Milestone: 5

Depends on: 0414, 0415, 0475

## Outcome

Recorded cases cover the real queries that went wrong for users and the reproduced AI Tumor Board case. Each case asserts against the source's own count or record, not against BioMCP's earlier output:

- AstraZeneca EGFR inhibitor trials started since 2024;
- NCI sarcoma filters that once returned 0 of 101 trials;
- an age-filtered melanoma trial count that once returned 20 against a true 472;
- MARIPOSA-2, amivantamab after osimertinib;
- PD-L1 prognosis in EGFR-mutant lung cancer articles;
- the AI Tumor Board EGFR L858R case as an acceptance fixture;
- eligibility text where a page break splits a criterion from its exception, so the continuation step is named;
- two variants on one allele that stay one record.

- Why: Ian's notes on real user failures (m13050, 2026-10-28) and the patient research findings of 2026-10-27.
- Starts from: the existing case layout under `tests/cases`, bounded live captures, and each source's own count for the same query.
- Rules: a case whose correct answer needs a capability 1.0 does not have, such as broadening an exact KRAS search to RAS, records the gap as a deferred ticket instead of a passing case. No clinical interpretation; a clinical reviewer chooses the expected records where judgment is needed.
- Proof: each case fails on the build that showed the user's failure where that build is still available, or on a planted wrong filter otherwise.
- Changes: cases and captures, and fixes they find.
- Defers: trials found only on hospital pages or non-US registries, and protocol-only thresholds such as KarMMa-1's organ function criteria, until after 1.0.
