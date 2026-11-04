# 0417 — Accept every trial status and keep a page on one bad value

Status: COMPLETE.
Landed: 2573be55ba803f1434f8d98054209d63c7c77005

Milestone: 5

## Outcome

`biodata::TrialStatus` includes ClinicalTrials.gov's `TEMPORARILY_NOT_AVAILABLE`, checked against the registry's published enum. A ClinicalTrials.gov search page or record with an unknown status, date or other non-identity value keeps the record with a problem, and only a bad NCT ID fails that record.

- Why: the pre-1.0 code quality review `sdlc/records/2026-10-28-code-quality-review.md`. 38 studies carry the missing status today, and any page holding one fails whole.
- Starts from: `crates/biodata/src/trial.rs`, `crates/biomcp-adapters/src/ctgov/raw.rs` and `detail.rs`, the shared `mapped` helpers.
- Rules: follow the shared bad-value rule; the status enum comes from the registry's published list as 0361 requires.
- Proof: a replay case with one study per missing status and one bad `overallStatus` in a page returns every other row with a problem; this priority case lands first.
- Changes: `crates/biodata/src/trial.rs`, `crates/biomcp-adapters/src/ctgov`, schemas, baseline and cases.
- Defers: nothing.
