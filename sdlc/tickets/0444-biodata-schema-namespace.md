# 0444 — Move BioMCP-owned types out of the biodata schema namespace

Status: COMPLETE.
Landed: 29b68fadcd62f0e8343a78bfa887f0c04d037c3d

Milestone: 5

## Outcome

Schema IDs start with `biodata.` exactly when the Rust type lives in the biodata crate. About 29 BioMCP queries and envelope parts, such as `biodata.article-search/1`, `biodata.request/27`, `Reason`, `Problem`, `ProblemCode` and `GeneResolution`, move to `biomcp.`. Every ID uses hyphens, and the underscore check covers both namespaces.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 9. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biomcp-types/src/grammar/queries.rs:41-58`, `schemas.rs:102` and `:321`, `runner/schema_proof.rs:176`, `biodata.article_header/3` and `biodata.trial_header/1`.
- Rules: no release has shipped, so the IDs move before the tag with no successor record. Coordinate with 0421, which also touches schema IDs.
- Proof: a check that a schema ID starts with `biodata.` exactly when its Rust type lives in the biodata crate.
- Changes: the IDs, the check and the baseline.
- Defers: nothing.
