# Interim schema baseline re-export for rehearsals

Status: OPEN.
Milestone: 5

## Outcome

`release/schema-baseline.*` is re-exported from main's own source now, so the private rehearsal's Linux x64 `schema-release-check` passes and the rehearsal reaches staging, publication, install and promotion. The release candidate ticket still re-exports it again as main's last commit before the merge.

- Why: private rehearsal run 41376870296 at 0fea901b failed Linux x64 `schema-release-check` with "request: schema identity changed or major version decreased" (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41376870296/linux-x64-job.log` lines 9631-9643). The baseline was exported on 2026-10-28 (ad126e42d) with `biodata.request/27`. The BioData schema namespace move (0444, 72351bdc5, later that day) renamed it, and main now generates `biomcp.request/32`. This is the planned baseline work, not a product bug: the 1.0 comparison compares 1.0 with itself by design (0336, release review C7). But until the baseline moves, every rehearsal stops Linux x64 before the downstream stages, which then go untested.
- Starts from: `scripts/schema_release.py --generate`, `release/schema-baseline.json`, `release/schema-baseline.tar.gz`, the 0336 baseline rules.
- Rules: export from main's source with the existing generator; mark the status as an interim candidate baseline, not the 1.0 baseline. Leave `release/migration-schema-baseline.tar.gz` and the release notes' migration list unchanged. Change no schema and no checker.
- Proof: `make schema-release-check` passes on the new head, the migration baseline bytes are unchanged, and the next private rehearsal's Linux x64 job passes this step.
- Changes: the two baseline files only.
- Defers: the final re-export, which 0336 still owns.
