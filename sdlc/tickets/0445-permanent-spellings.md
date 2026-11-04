# 0445 — Fix permanent spellings before the tag

Status: COMPLETE.
Landed: 2f198f3e747bdc5382c0a3c1ced3f338e80c8e21

Milestone: 5

## Outcome

PGx uses `annotation_kind`, not `annotationkind`. Semantic Scholar has one spelling across `ArticleKey`, `Source` and the `AuthorId` pattern. BioMCP-owned values use one case style; source-published values keep their source spelling.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 12. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biomcp-types/src/pgx/response.rs:118`, `ArticleKey`, `Source`, `AuthorId` and `Reason`.
- Rules: the case style follows the existing `Source` kebab-case unless the schema ID convention says otherwise. Namespaced error codes such as `usage.mcp_unavailable`, `usage.pool_mismatch`, `usage.unsupported_filter` and `wire_enum.unknown` keep their dotted snake_case convention and do not change here; 0426 owns their contract, and the lint exempts the error-code type rather than a list of names (m13890). BioData wire values, including the statement vocabulary (`Code`, `Predicate` and `Coding.system` such as `oncokb_oncogenicity`), stay snake_case, because Ian chose snake_case as the native wire spelling and the shared BioData plan (section 5, rule 2) fixes it with Genome; the kebab-case rule applies only to BioMCP-owned values outside `crates/biodata`, where `Source` sets the style (m13911).
- Proof: a lint over serde tag names and BioMCP-owned enum values.
- Changes: the named types, schemas, cases and the baseline.
- Defers: nothing.
