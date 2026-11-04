# 0453 — Add a BioData value test sweep

Status: COMPLETE.
Landed: dde31fc8dab5fb0da51d52eb80c221b684e75f82

Milestone: 5

## Outcome

For every public biodata type with a text form, `parse(display(v)) == v`, serialize then deserialize gives an equal value, and deserializing runs the same checks as construction, with near-miss rows per type. Equal values hash equally, including `GeneSymbol` and `Doi` case variants and `ScientificPValue` `1.50e-8` against `1.5e-8`. A `proptest` property test on the `FromStr` of `Accession`, `HgvsText` and `VariantInput` asserts no panic and the round trip. All run in the routine gate.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 21. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata` and `crates/biomcp-types/tests`.
- Rules: `proptest` is approved as a dev-dependency (PM ruling 2026-10-28). The forward-compatibility, pattern and read-back tests stay in 0437, 0440 and 0439.
- Proof: a planted broken round trip and a planted unequal hash fail the sweep.
- Changes: tests only, plus fixes for what they find.
- Defers: nothing.
