# 0409 — Find NCI trials at a named organization

Status: COMPLETE.
Landed: eea7d3f559c050a9ac157b598681820112bd306e

Milestone: 5

Depends on: 0408, 0414

## Outcome

`biomcp search trial --source nci --facility <exact NCI organization name>` returns NCI trials with a site of that name, using NCI's `sites.org_name` filter (exact, case-insensitive). Each trial header carries one matched site with its name, city, state and native recruitment status, plus the number of matching sites. An organization header from 0408 names this search as its next step. Help says a listed site is not necessarily enrolling.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. It takes an agent from a named cancer center to its trials.
- Starts from: 0408, `crates/biomcp-types/src/query/trial.rs` (NCI refuses `--facility` today) and NCI's documented `sites.org_name` field variants.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail. Capture first whether NCI's public tier returns `sites` with `include=sites`; if not, drop the matched-site field and echo the request only. A returned trial with no locally equal site name keeps its place with a problem. Combining `--facility` with other site filters on NCI refuses, because NCI does not document that they bind one site. The ClinicalTrials.gov `--facility` behavior is unchanged.
- Proof: a case for a known organization asserts at least one trial, and the trial total equals the 0408 count for the same name; cases for the no-match problem and the refusal; header bytes measured on a trial with many sites.
- Changes: `crates/biomcp-types/src/query/trial.rs`, `crates/biomcp-adapters/src/nci`, the organization next step, help and cases.
- Defers: NCI's stemmed and prefix name matching.
