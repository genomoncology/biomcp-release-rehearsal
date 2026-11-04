# cBioPortal studies return mutation and sample rows

Status: OPEN.
Milestone: 5

## Outcome

`get study` adds `mutations` (filtered by gene, with sample ID, protein change and variant class) and `samples` sections that page through rows from cBioPortal, not only counts and file links.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 6. `variant recurrence` gives an exact-allele count per page and `get study ... files` gives links, but no rows; cBioPortal's `mutations/fetch` and `/studies/{id}/samples` return them. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-adapters/src/cbioportal/`, `crates/biomcp-query/src/study/`, and the study types in `crates/biomcp-types/src/study/`.
- Rules: cBioPortal's public API only, already called; rows page with `--section-limit` and `--section-offset`; `mutations` requires `--gene` and a molecular profile, defaulting to the study's mutation profile with the choice shown. No clinical attributes beyond sample ID and type.
- Proof: installed-binary cases: BRAF mutations in `skcm_tcga_pan_can_atlas_2018` include V600E rows with sample IDs; samples page with consistent totals.
- Changes: `crates/biomcp-adapters/src/cbioportal/**`, `crates/biomcp-query/src/study/**`, `crates/biomcp-types/src/study/**`, `crates/biomcp-types/schemas/study*.json`, `tests/cases/study/**`, `tests/replies/study/**`; study sections, the cBioPortal requests, schema, help, map, cases.
- Defers: nothing.
