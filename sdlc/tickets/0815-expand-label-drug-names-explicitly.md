# 0815 — Add disclosed bounded label-name expansion

Status: OPEN.

Milestone: 5

## Outcome

Drug search on openfda-label offers explicit bounded drug-name expansion, preserving literal exact-name-first default. It searches one checked native union, discloses aliases/uncertainty, and freezes aliases plus matching tier in continuations. Distinct SPL sets and source revision metadata remain independent.

## Why

Ian requested addressing label synonym coverage. Retained verified34d persona replies show literal apixaban total1 versus ELIQUIS total8; current discovery stops at its first nonempty exact tier and after-empty alias recovery never runs. This is an explicit discovery feature gap, not proven cache staleness, obsolete labels or a violated literal contract.

## Proof

- CLI/MCP controlled results include a nonempty literal match and an additional brand match; default remains literal while explicit expansion finds the native union. Overlapping aliases do not sum counts or duplicate native membership.
- Preserve indication filters, native order/totals, separate SPL IDs/version dates and exact get steps. Never compare revision numbers globally across sets or synthesize a universal current label.
- Cover complete/ambiguous/incomplete/failed resolution, source/component exclusions, configured name/query bounds and escaping. Primary label failure remains failed despite resolver success.
- Follow emitted continuations after resolver fixture changes: aliases and tier stay frozen; tampered/missing/disallowed binding gives explicit restart, no hidden re-resolution. Cover exhausted native pages.
- Run focused owning fixtures, schema/help checks and routine gate. No live provider call or full release rebuild during development proof.

## Starts from

- openfda_label/search.rs discover stops at first nonzero brand/generic exact tier; drug/aliases.rs only resolves after an empty result. DrugSearch documents exact-first behavior. Existing complete-evidence drug resolver and caller-authorized source_snapshot storage should own reused policy/state; do not copy FAERS adapter/query code.
- Retained persona evidence worktrees/bench-persona-pt-committee/personas/pt-committee from verified34dLinuxwheel: apixaban set095a08ac version7/revised2021-06-15, ELIQUIS includes seta454cd24 version38/revised2026-04-24. Trimmed envelopes lack native harmonized fields, so the particular field/tier cause remains unproved.

## Keeps

- Explicit opt-in only, literal default and existing zero-result recovery outside expansion mode. One bounded escaped native OR over searched brand/generic spellings AND unchanged indication filter; native totals, never added per-name counts.
- Resolver completeness/ambiguity, budgets/name caps, source/component exclusions and caller authorization before saved payload access. Freeze selected tier as well as names; bypass legacy after-empty alias path in explicit mode to avoid inconsistent second resolution.
- Application-number searches, exact SPL reads, literal indication wording and source attribution stay exact. No new INN ontology inference, merged SPL sets, automatic source/tool download or dependency.

## Defers

- Indication synonym expansion and native substance/INN-field research need separate evidence. Preserve current qualification identities and assess affected ending-candidate proof after landing accepted fixes.

## Changes

- Checked drug-search expansion option and retained continuation state, label search/resolver integration, disclosure/help and CLI/MCP fixtures. Reuse shared policy without a new query framework.
