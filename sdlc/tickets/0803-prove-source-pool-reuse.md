# 0803 — Prove healthy source-pool reuse and incomplete membership recovery

Status: OPEN.

Milestone: 5

## Outcome

Outside-in regressions prove that healthy author/protein/pathway source windows remain reusable defaults and incomplete author/protein-search/pathway observations remain explicitly readable without becoming defaults, including after a native author extension.

## Why

The bounded mutation sample on product 34d found six meaningful survivors despite all existing cases passing: protein tuple completeness forced false, protein-search completeness forced true, and Semantic Scholar author and Reactome pathway completeness forced true or false. Independent inspection confirmed Medium regression gaps, not defects in the unmutated runtime.

## Proof

- Repeat an identical healthy author search without a pool selector; require stable pool ID and records, including when later native pages remain unread.
- Supply HTTP 200 author results containing valid and refused identities. Require a partial reply, stable explicit pages, and a subsequent identical implicit search that does not reuse the incomplete pool.
- Extend a healthy current author window through its emitted continuation into a mixed valid/refused page. Repeating the original implicit search must select a new pool, while the explicit incomplete observation remains readable.
- Extend the existing protein section boundary proof with an identical no-pool repeat; require stable pool ID and records, retaining explicit continuation as the control.
- Supply HTTP 200 UniProt search results containing admitted and refused rows while source state remains Ok. Require a partial reply with stable followable explicit pages, and require an identical implicit repeat without --fresh to create a different pool. Retain the original explicit observation as the control; use refused membership, not an HTTP failure or optional-field problem.
- Repeat identical healthy pathway events commands without a pool selector; require complete replies, stable pool ID and records, retaining explicit continuation as the control.
- Serve a valid pathway card and HTTP 200 events with two admitted events plus one invalid Reactome identity. At limit 1 require a partial reply and followable explicit continuation with stable pool identity. The identical implicit command must create a different pool while the original explicit pages remain readable. Do not substitute optional-field trouble or an HTTP failure for refused membership.
- Prove these public-command checks fail under the six meaningful replacements and pass unmutated. Run focused checks and the routine gate; preserve red/green evidence and actual test-driver identity.

## Starts from

- Product 34d6e02d1ebeac4d55e44e05099854f15e718d57, current main and existing fixture/stub runners. Protein retention is crates/biomcp-query/src/protein/relations/retained.rs; healthy entry admission is crates/biomcp-adapters/src/uniprot/entry/map.rs. Author refusal retention is crates/biomcp-query/src/author/search.rs and shared publication is crates/biomcp-query/src/source_snapshot.rs. Existing protein boundary proof: crates/biomcp/tests/runner/protein_sections/boundaries.rs.
- Native survivor logs/diffs are receiver-verified on Yellow under data/biomcp-redux-evidence/0788/j36944-{second,third,fourth,fifth}-survivor/. The independent assessment traced tuple-true publication to accepted UniProt entries; do not invent an arbitrary tuple assertion to kill that masked survivor.
- Pathway witnesses are j36944-sixth-survivor and j36944-seventh-survivor in the same evidence store. Reactome relation admission can retain valid rows alongside refused identities while source state stays Ok; pathway/relation.rs retains dropped membership. Extend the existing crates/biomcp/tests/runner/pathway_sections.rs runner.
- Protein-search witness j36944-eighth-survivor covers crates/biomcp-query/src/protein/search.rs Saved::whole forced true. UniProt search can retain admitted and refused membership with source state Ok; the shared source-status check does not mask this mutation. Its false replacement was caught by the original sample.

## Keeps

- Production code, schemas, mutation selection, exclusions, budgets and survivor scores. No --fresh on implicit repeats. Pool identity and records are decisive; raw request counts alone cannot distinguish reuse from reconstruction hidden by HTTP caching. If checking repaired provider data, expire only raw transport fixtures while retaining query pools.
- The existing bounded shard finishes under its original identity/deadline; no retry to obtain green. Supplementary regression proofs name their real test-driver source and demonstrate production equivalence to selected product 34d.

## Defers

- Final mutation collection and qualification disposition remain in 0788. Article overbroad migration invalidation is Low/nonblocking; tuple-true remains nonblocking under the traced production invariant, not universally equivalent. Preserve both survivors without exclusions or mirrored implementation tests.

## Changes

- Existing CLI fixtures, stub responses and owning author/protein/pathway integration tests. No runtime changes expected; report a genuine baseline defect before broadening scope.
