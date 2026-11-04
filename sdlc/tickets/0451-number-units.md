# 0451 — Carry a unit with each number whose source states one

Status: COMPLETE.
Landed: 2be3bb5367ee16c0c5e530af1c9dfc6e8f40c619

Milestone: 5

## Outcome

Where the source states a unit, a number and its unit travel as one value: `StatisticEstimate`, the expression abundance values (`ntpm`, `ptpm`) and the NCI and FAERS age fields.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 18. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: the named types and adapters.
- Rules: no unit is inferred where the source states none.
- Proof: a recorded NCI age reads as a number plus a unit.
- Changes: the named types, adapters, schemas and the baseline.
- Defers: nothing.
