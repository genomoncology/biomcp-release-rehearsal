# 0437 — Publish open reply schemas and make the gate protect readers

Status: COMPLETE.
Landed: 3ef45d0eca68735829fe08f6c7d7e827c5d3e724

Milestone: 5

## Outcome

Reply and value schemas are generated without `additionalProperties: false`, and reply types are read without `deny_unknown_fields`. Query schemas stay closed. The `biomcp schema` root description and the docs state that readers ignore unknown fields and unknown enum values within a major, and growable enums such as `Source`, `Reason`, `Proposition` and `Predicate` carry an `x-` annotation saying so. The compatibility gate marks each contract as query or reply: for replies, removing a name from the required set takes a major, a reviewed pattern widening recorded beside the baseline is allowed, and a declared successor schema ID is allowed at a new major.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 3 and 10. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biomcp-types/schemas`, the schema generator, the reply type derives and `crates/biomcp-types/tests/schemas/compatibility.rs:35`, `:54`, `:67` and `:68`.
- Rules: `architecture.md` near line 1057 already says a removed required reply field takes a major; the gate now enforces it.
- Rules: PM ruling 2026-10-28 on unknown enum values. The forward rule is a promise to JSON readers. A growable enum's schema accepts any string beyond its known values, with the known list kept under the `x-` annotation. Rust types keep closed enums with no `Other(String)`, as the architecture says, and refuse an unknown value; Rust readers read replies of their own version, because the biodata Rust API is unpublished. Rust reply types still drop `deny_unknown_fields`.
- Proof: a recorded reply with an added property at `/result` and at one nested value and an unknown enum value validates against the baseline schema with a `jsonschema` dev-dependency (approved); the same reply with only the added properties reads into an existing deserializable type such as `SourceOutcome`; full envelope readback stays in 0439, so 0437 does not wait for it; a planted reply change that removes one required name fails the gate.
- Changes: the generator, derives, gate, docs and baseline.
- Defers: nothing.
