# 0813 — Admit recurrence sample sets before counting mutations

Status: COMPLETE.
Landed: c17004a7740ce61be8923e5f129e462276dea11f

Milestone: 5

## Outcome

Recurrence requires a successfully admitted cBioPortal sample list with the requested study/list identities before fetching or counting mutations. Unknown, malformed or mismatched lists cannot yield observed/examined zero or an all-pages-read result. A valid admitted list may still have an optional unknown sampleCount.

## Why

Code inspection confirms cohort admission catches every list error as optional count_error and continues fetching mutations. Unknown-list404 plus mutation200[] can produce zero counts despite later source-failed/overall-partial diagnostics. The persona wording 'no error' is unproved, but an invalid cohort cannot establish zero recurrence. Ian asked to prioritize this finding.

## Proof

- Through CLI and MCP, unknown-list404 with a mutation endpoint configured200[] must fail without observed result/counts and make zero mutation requests.
- Wrong study/list identity and malformed list identity must refuse before mutation access.
- Valid list plus empty mutation page remains a complete true zero with examined0/all-pages-read; valid list without sampleCount remains usable, omits an invented denominator and discloses missing count.
- Preserve nonzero recurrence and paging controls. Prove new cases fail on old behavior and pass fixed, using existing recurrence/admission and cohort fixtures, focused pm jobs and routine gate.

## Starts from

- crates/biomcp-adapters/src/cbioportal/mod.rs cohort lookup near90 and crates/biomcp-query/src/variant/recurrence.rs later metadata handling. Existing runner recurrence/admission.rs and cohort.rs contain true-zero and missing-count controls.
- Retained valid persona envelope worktrees/bench-persona-tumor-board/personas/tumor-board/evidence/q5-recurrence-pik3ca.json reports128observed/355examined; unknown --sample-set all is log-only. Do not claim raw failure evidence was retained when it was not.

## Keeps

- Native list/study/profile identity, paging, observed counts and optional denominator semantics; source/caller rights, budgets and diagnostics. Missing optional count is not missing list admission. No clinical inference, guessed denominator or broad metadata-failure swallowing.

## Defers

- Other recurrence statistics or new cohort semantics are out of scope. Preserve existing qualification under actual source and assess affected proof after accepted fixes land.

## Changes

- cBioPortal cohort identity admission and owning recurrence CLI/MCP regression fixtures. No new source or dependency.
