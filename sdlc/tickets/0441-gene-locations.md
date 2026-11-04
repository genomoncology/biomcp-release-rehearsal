# 0441 — Keep gene locations that HGNC states in other forms

Status: COMPLETE.
Landed: 3cd3429ef3e8bba7f7e73bd199a2cd3e1c56bcae

Milestone: 5

## Outcome

A gene location holds one or more cytobands and the mitochondrial genome, and keeps HGNC's text when it does not parse. SHOX (`Xp22.33 and Yp11.32`), CRLF2 and MT-ND1 (`mitochondria`) come back with a location and no problem.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 5. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata/src/id/gene.rs:164` and `crates/biomcp-adapters/src/hgnc/map.rs:54`.
- Rules: range forms are checked against the HGNC data set and kept.
- Proof: HGNC fixture rows for SHOX, CRLF2, MT-ND1 and a range form, checked through `get gene`.
- Changes: the location type, the HGNC mapping, schemas, cases and the baseline.
- Defers: nothing.
