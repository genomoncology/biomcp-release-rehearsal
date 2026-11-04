# 0446 — Finish PopulationFrequency

Status: COMPLETE.
Landed: 63af932a6cb46518234e78e401e5edc738651a4a

Milestone: 5

## Outcome

`allele_frequency` and `faf95` reference one `Fraction` definition, finite from 0 to 1, that the schema states. `PopulationFrequency` gains the optional union fields Genome carries: `heterozygote_count`, `grpmax_group`, `genotype_filter` and homoplasmic and heteroplasmic counts, as the shared plan's one-name-per-concept rule requires.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 13. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata/src/variant/population.rs:31`, `:38` and `:57`, and `schemas/population-frequency.json`.
- Rules: fields are filled only where a source publishes them.
- Proof: a schema assertion of minimum 0 and maximum 1 at both paths, and a refusal case for 1.5.
- Changes: the type, adapters that publish the new fields, schemas and the baseline.
- Defers: nothing.
