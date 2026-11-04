# GWAS studies are found from their paper, and large association pages finish

Status: OPEN.
Milestone: 5

## Outcome

`search gwas --pubmed <PMID>` can return the studies of that paper (study accession, trait, sample and association count) as well as associations, and `search gwas --pubmed ... --limit 50` finishes within the command budget.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 7. Only association rows are returned for a PMID; GWAS Catalog v2 lists 7 studies for PMID 17554300. `--limit 50` failed with exit 4, deadline exceeded. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-adapters/src/gwas/`, `crates/biomcp-query/src/gwas/`, and `crates/biomcp-types/src/gwas/`.
- Rules: a study-level result mode chosen by an explicit option, the association mode unchanged by default. Find the general cause of the deadline (per-row follow-up requests or a serial fan-out) and fix it there, not by raising the budget.
- Proof: installed-binary cases: the witness PMID returns its 7 studies; `--limit 50` associations answer within the default budget; existing GWAS cases unchanged.
- Changes: `crates/biomcp-adapters/src/gwas/**`, `crates/biomcp-query/src/gwas/**`, `crates/biomcp-types/src/gwas/**`, `crates/biomcp/src/grammar/gwas.rs`, `tests/cases/gwas/**`, `tests/replies/gwas/**`, `tests/gwas-*-budgets.yaml`; gWAS search, requests, schema, help, cases.
- Defers: nothing.
