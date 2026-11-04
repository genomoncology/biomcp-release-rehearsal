# 0443 — Omit nulls from JSON output

Status: COMPLETE.
Landed: 3b921a9dacfc8420f14adeb6bc8840221e8ad0fe

Milestone: 5

## Outcome

Every `Option` field skips `None`, as the 2026-10-23 JSON output decision says, unless absence and null carry different meanings, in which case the field gets an explicit value. About 85 fields change, including `trial_detail.rs:27` sponsor, `trial_sites.rs`, `nci_detail.rs` and GWAS `qualifying_total`.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 7. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: the fields above and `tests/cases/gwas/examples.yaml`.
- Rules: `aggregator_knowledge_source` keeps its documented missing, null and list distinction.
- Proof: a `reject_nulls` check beside the runner's empty-list check in `runner/schema.rs:84`; it fails today on the GWAS case.
- Changes: the fields, the check, cases and the baseline.
- Defers: nothing.
