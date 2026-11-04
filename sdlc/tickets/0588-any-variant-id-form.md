# Every variant command accepts every variant ID form

Status: COMPLETE.
Landed: f375d12e0a5b03aa956003d9af60cdaf09ea75f0
Milestone: 5

## Outcome

Every variant command resolves any supported variant ID form through one shared resolver: ClinVar variation IDs (`clinvar:N` and `VCV…`), rsIDs, HGVS, and genomic coordinates such as `7-140753336-A-T`. When a form cannot serve a command, the error names the forms that can and the command that converts them.

- Why: experiment 513 (2026-11-03) found this the single worst trap; one task cost 2.5 times as much because of it. `get variant` rejects `VCV…` and gnomAD-style IDs; `variant assertions --source clinvar` accepts only `clinvar:N`, not an rsID or HGVS; `variant population clinvar:13961` fails with "no GRCh38 allele". The benchmark of 2026-11-02 also found rsIDs default to GRCh37. Ian approved fixing the general causes before 1.0 (2026-11-03).
- Starts from: `crates/biomcp-query/src/variant/` (`population_identity.rs`, `assertions.rs`, `gwas.rs`) and the variant identity types in `crates/biomcp-types/src/response/variant.rs`.
- Rules: one resolver, used by every variant command; no per-command parsing. Resolution reuses the sources BioMCP already calls. An rsID resolves on GRCh38 by default, with the assembly as a setting. A ClinVar record without a GRCh38 allele falls back to its other alleles and says so in the reply. No clinical choice of examples beyond public IDs already in the cases.
- Proof: an installed-binary case matrix of each ID form against each variant command, recorded, with every cell either answering or giving the named-forms error; the three witness commands above succeed.
- Changes: the variant resolver, the variant commands, help text naming the forms, and cases.
- Defers: nothing.
