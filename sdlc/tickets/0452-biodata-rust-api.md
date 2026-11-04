# 0452 — Fill the Rust API gaps an SDK needs

Status: COMPLETE.
Landed: a0aa051467531e0c1bcece8b334fc6d7073f3725

Milestone: 5

## Outcome

The biodata Rust API gives getters on `VariantRef`, `PopulationFrequency` and `TrialCoordinates`, `Expression`'s syntax version, and text access on `Doi`, `NctId`, `Curie` and `VcvId`; `Hash` on `VariantId`, `Accession`, `HgvsText`, `Expression`, `Chromosome` and the `wire_enum!` types; `FromStr` and `Display` on the `wire_enum!` types. `VcvId`, `RcvId` and `ScvId` compare by identity, so `VCV000013961` and `VCV000013961.3` relate. `Accession` fields become private.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 19. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata`.
- Rules: the `non_exhaustive` choice for `TrialStatus`, `TrialPhase` and `StudyType` follows the restructuring design's ruling on internal enums.
- Proof: a compile-time API witness in `biomcp-types/tests` that builds an EGFR c.2573T>G value, selects its coding representation, reads each statement subject and population frequency, and puts every ID type in a `HashSet`.
- Changes: crates/biodata and the witness.
- Defers: nothing.
