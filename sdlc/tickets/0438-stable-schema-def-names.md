# 0438 — Give schema definitions stable names

Status: COMPLETE.
Landed: bbf1c3ad59ad1ff7da211b12c15215a14f7d38d5

Milestone: 5

## Outcome

Generic types get explicit schemars names, such as `Section_{T}`, so no `$defs` name carries different bodies in different files. Today `Section` has 18 bodies and `Section2` has 11.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 11. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: the generic response types and the checked-in schemas.
- Rules: names follow the hyphen and case rules of the schema ID convention where they apply.
- Proof: a check across all checked-in schemas that each `$defs` name maps to one body.
- Changes: the generic types, the check and the baseline.
- Defers: nothing.
