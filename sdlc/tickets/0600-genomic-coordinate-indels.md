# Genomic coordinates accept insertions and deletions

Status: COMPLETE.
Landed: 7cfabaf7ea50306b740dad4725cdaed3d968bdea
Milestone: 5

## Outcome

Every variant command accepts gnomAD-style coordinates whose reference and alternate differ in length, such as `13-32315508-GT-G` or `chr1-55051215-G-GA`, through the shared variant resolver.

- Why: `planning/eval-recommendations-2026-11-04.md` item 2 lists "any variant ID form, including gnomAD-style indels" in the bundle that beat the cold agent. Main accepts only single-base coordinates: `COORDINATE_PATTERN` in `crates/biomcp-types/src/query/variant_input.rs` requires one base each side and refuses the rest with "Multi-base coordinates are unsupported", and `crates/biomcp-types/tests/variant_values.rs` asserts that refusal. gnomAD, VCF and most papers write indels this way.
- Starts from: `COORDINATE_PATTERN` and `coordinates()` in `variant_input.rs`, the shared resolver from 0588, and its case matrix.
- Rules: coordinates are read as VCF style on GRCh38, the assembly setting from 0588 applying. A shared leading anchor base is trimmed, and the result becomes genomic HGVS (deletion, insertion, duplication only where the source confirms it, or delins). The reference bases are checked against a source BioMCP already calls before any answer; a mismatch is an error naming the expected base. No new source.
- Proof: the 0588 installed-binary matrix gains a deletion, an insertion and a delins row for every variant command, each answering or giving the named-forms error; a wrong reference base is refused; the single-base cases are unchanged.
- Changes: `crates/biomcp-types/src/query/variant_input.rs`, `crates/biomcp-types/tests/variant_values.rs`, `crates/biomcp-query/src/variant/**`, `tests/cases/variant/**`, `tests/replies/variant/**`; `variant_input.rs`, the resolver, its tests (the refusal test becomes acceptance), help text naming the form, and cases.
- Defers: nothing.
