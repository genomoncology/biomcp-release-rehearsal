# 0447 — Let a term carry several codings and keep CURIEs apart from URIs

Status: COMPLETE.
Landed: e2300e6a95d8b7fb67be5d1fc196ec310104661e

Milestone: 5

## Outcome

`Term` holds a list of codings. The `Curie` pattern accepts only `prefix:local`, so a system URI or `urn:oid:` value is refused as a concept ID. ClinVar review status and allele origin go out coded as `clinvar_review_status` and `clinvar_allele_origin`, with Genome's spellings such as `criteria_provided_single_submitter` and `de_novo`.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 14. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata/src/variant/statement/coding.rs:4`, `statement/mod.rs:58` and `:64`, and `crates/biomcp-adapters/src/assertion.rs:29` and `:38`.
- Rules: check every recorded reply before tightening `Curie`. Open codings with system URI, version and display wait for 1.1 in new fields.
- Proof: a `Term` with two codings round-trips through Rust and validates, and a URI in `Concept.id` is refused.
- Changes: the named types, adapters, schemas and the baseline.
- Defers: nothing.
