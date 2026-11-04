# 0448 — Use one identifier and authority shape

Status: COMPLETE.
Landed: 2e4be4f5f30ce9a1dcae2eb422e380b31b5a3ddd

Milestone: 5

## Outcome

Every identifier object uses one property pair, `namespace` and `id`, with an explicit unknown value where the authority is not stated. This covers `DrugDiseaseReference`, `PathwaySourceRef`, `OrcidExternalId`, `ArticleKey`, `DrugIdentity`, `SourceRecord` and `GwasTrait`. Raw integer IDs such as `CivicGene.id`, `GwasAssociation.id`, the ClinPGx IDs and Reactome `db_id` carry their authority, and the GWAS association ID reads the same in replies and in the get query.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 15. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: the named types and `gene/civic.rs:9`, `variant_population.rs:70`.
- Rules: `ArticleKey` keeps its existing spelling if it already matches.
- Proof: a schema lint that requires identifier objects to use the agreed names.
- Changes: the named types, adapters, schemas, cases and the baseline.
- Defers: nothing.
