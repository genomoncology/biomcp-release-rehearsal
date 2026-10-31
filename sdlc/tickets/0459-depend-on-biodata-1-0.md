# 0459 — Depend on the extracted BioData 1.0 crate

Status: OPEN.

Milestone: 5

Depends on: 0432, 0517

## Outcome

BioMCP builds against the BioData 1.0 crate extracted from this repository's `crates/biodata`, pinned to an exact published revision, and the in-tree copy is removed. The full gate and the schema comparison pass with every `biodata.*` schema byte-identical to the last in-tree export except listed optional additions and listed added enum cases, so the switch changes no reply.

- Why: Ian's ruling of 2026-10-28 that BioMCP 1.0 ships depending on BioData 1.0, not an in-tree copy, and that this repository's 1.0 schemas and `biodata.*` IDs are BioData's baseline.
- Starts from: `crates/biodata`, the workspace manifest, `Cargo.lock`, `deny.toml`, the release build scripts and the schema baseline.
- Rules: the signal to extract is a PM message giving the main commit after every code ticket above has landed; from that commit until this ticket lands, no change touches `crates/biodata`. The BioData side owns extraction and reconciliation with Genome. A difference in any `biodata.*` schema or reply is a BioData fix, not a BioMCP baseline change. The dependency must resolve on clean hosted runners and in the offline release build, with the source policy in `deny.toml` allowing only the pinned source. Creating the BioData repository and publishing its packages wait for Ian.
- Reconciliation with BioData (m13084, 2026-10-28): BioMCP pins `github.com/genomoncology/biodata` by tag and exact commit, and the offline release build vendors it with `cargo vendor`. BioMCP enables BioData's `schema` feature, which turns on serde and schemars. Renames keep their schema names: `GeneRef` becomes `SourceGeneRef` and `FiniteNumberError` becomes `NumberError`. BioMCP adds no wrapper types: where Genome's shape is wider, BioData's type carries BioMCP's wire with Genome's extra fields as optional additions, so replies stay byte-identical and any `biodata.*` schema change is an optional addition the reconciliation record lists. BioMCP-owned schema IDs have already moved to `biomcp.*` in 0444.
- Added enum cases (m13513, 2026-10-29): Genome's wider shapes add enum cases as well as optional fields. `Statement.subject` becomes the shared plan's `Subject` (section 3.6), `VariantRef` gains the categorical case, `Proposition` gains the four gene propositions and `Coding` gains seven systems. The schema comparison accepts an added case as a listed addition when BioMCP never emits it and every existing case serializes byte-identically; a removed or renamed case, or a changed existing case, fails. This ticket makes the BioMCP code change from `VariantRef` to `Subject` through BioData's `From<VariantRef> for Subject`, because nothing touches `crates/biodata` after the signal commit.
- Narrowed ID patterns and versioned ClinVar equality (m17794, 2026-10-31): the schema comparison also accepts four listed narrowings. The VcvId, RcvId and ScvId version and the CosmicId number shrink to BioData's stored integer range, because the 1.0 baseline is unpublished until 0336 re-exports it. Each narrowed pattern accepts exactly what its parser accepts, and a near-miss table proves that at the bound. Any other pattern change still fails. ClinVar ID equality and hashing include the version. Wherever BioMCP relied on version-blind equality, it uses BioData's named version-blind method. The reply diff proves no reply changes.
- Proof: a reply diff between the last in-tree build and the switched build is empty; the schema comparison passes; the hosted rehearsal resolves the dependency on every platform.
- Changes: the manifests, lock file, source policy, release scripts and removal of `crates/biodata`.
- Defers: FHIR and OMOP, which land in BioData and BioMCP after 1.0.
