# 0439 — Make every reply readable back into its types

Status: COMPLETE.
Landed: 269950823b926494e620c8f34eb1a13501ff5ce5

Milestone: 5

Depends on: 0437

## Outcome

`Envelope`, `Help`, `SchemaCatalog` and the GWAS get reply derive `Deserialize`. Reading an envelope checks that `schema` equals the body type's schema ID, and a registry maps each schema ID to its body type. Readers tolerate unknown fields as 0437 says.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 8. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biomcp-types/src/response/envelope.rs` and `gwas/get.rs:20`.
- Rules: no change to written JSON.
- Proof: every recorded case reply reads into its type, writes back and compares byte-equal.
- Changes: the named types, a registry and the test.
- Defers: nothing.
