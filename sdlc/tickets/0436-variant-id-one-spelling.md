# 0436 — Print each variant ID one way and accept every ID BioMCP prints

Status: COMPLETE.
Landed: 6aa9ab8ac10b075e524e02723821375b5d08b932

Milestone: 5

## Outcome

Every `VariantId` on the wire uses its published text, as `sdlc/planning/biodata-variants-and-annotations.md` section 3.4 says: `rs113488022`, `CA123643`, `PA…`, `VCV000013961`, `COSV…` and `ga4gh:VA…`. Only an ID whose published text is a bare number carries a namespace, as `clinvar:13961` does today. `Variant.ids`, statement subjects, population rows and references all use the same text, and `VariantInput` accepts every spelling BioMCP prints. The qualified forms that 0362 introduced, such as `rs:` and `clingen:`, go.

- Why: the BioData review of BioMCP 1.0's data model (2026-10-28), ask 2. Ian ruled that incomplete or incorrect 1.0 data model types are fixed before the tag, because after it the release gate freezes every pattern, type set and schema ID.
- Starts from: `crates/biodata/src/variant/output_ids.rs:21-26`, `statement/reference.rs:142`, `crates/biomcp-types/src/response/variant.rs:33`, `variant_population.rs:54`, `crates/biomcp-types/src/query/variant_input.rs`.
- Rules: PM ruling 2026-10-28: published text, because it is what each source and every other tool recognizes, it matches the shared plan and Genome already follows it. The ClinGen `PA` and ClinPGx `PA` collision does not arise, because a `VariantId` field never holds a ClinPGx accession.
- Proof: a CLI case that runs `get variant` on every ID in a search result's `identity.ids` and statement subjects; a golden card where one ID is spelled the same in ids, requested and subject.
- Changes: the named files, schemas with a major bump if the gate calls it breaking, cases and the baseline.
- Defers: nothing.
