# 0795 — Require complete evidence for drug name expansion

Status: COMPLETE.
Landed: bf181e7fb76267c7d39bebeb77e5d421fcd51704

Milestone: 5

## Outcome

Trial and article search expand a drug name only when a bounded source lookup establishes one structure from complete candidate evidence. Refused rows or an unfinished candidate window produce an incomplete-lookup notice and typed-term search when no other complete source resolves it.

## Why

The resolver ignores MyChem refused candidates and discards the native total before treating the first 50 candidates as a uniqueness proof. Evidence: `records/2026-11-10-product-review.md`, F3.

## Starts from

- `crates/biomcp-query/src/drug/search_names.rs` (`mychem`, `card`, `resolve`), `crates/biomcp-adapters/src/mychem/mod.rs` (`name_records`), `mychem/names.rs`, `crates/biomcp-query/src/article/drug_names.rs` and existing drug-name expansion CLI cases. `card` also treats a dropped result as no match, losing the incomplete/no-match distinction.

- PM decision, 2026-11-10: retain native total/received/refusal facts and prove completeness within the existing lookup budget and cap. A prefix with one observed structure is not proof of uniqueness; provider ranking is not a completeness guarantee. Try existing fallback sources within the same declared bounds. If none gives complete unique evidence, search the typed term and disclose the incomplete lookup; do not falsely call it ambiguous or absent. No unbounded scan, hand-maintained drug list or new source.

- Unresolved identity must remain a failed/incomplete resolver observation (`Resolved.failed`), including successful HTTP replies with capped candidates, so article current-pool admission cannot treat a notice alone as a whole answer. A complete unique fallback clears the lookup uncertainty. Reassess or invalidate legacy implicit expanded article pools that bypass the resolver through `article/drug_names.rs::reuse`; preserve explicitly addressed old pages and avoid a general cache purge.

- Approved implementation plan: revise only the current-key hash, derived from the existing exclusion-bound current hash plus a drug-resolution revision, for non-exact searches with a drug-name input. Keep the identity hash and explicit pages unchanged.

## Proof

- Routine installed replay cases for article and trial entry points: one good exact row plus a refused possibly matching row; a capped prefix with an unseen possible second structure; a complete fallback succeeding after the first source is incomplete; every source unresolved; and `--exact-term` making no resolver calls. Complete same-structure stereo/product rows still expand under the existing rule, and two known structures remain ambiguous. Assert sent search terms, notices and request bounds, not only result count. Repeat a no-pool article search after incomplete resolution and seed a legacy implicit expanded pool to prove both paths cannot bypass completeness checks.

## Keeps

- Source-derived names only, input and provider filters, exact-term escape hatch, publisher ranking, current name-count limit, admitted result/status semantics and licensed-component exclusions.

## Changes

- Changes: `crates/biomcp-query/src/drug/search_names.rs`, `crates/biomcp-query/src/article/drug_names.rs`, `crates/biomcp-query/src/article/mod.rs`, `crates/biomcp-adapters/src/mychem/mod.rs`, `crates/biomcp-adapters/src/mychem/names.rs`, `tests/cases/drug/`, `tests/cases/article/`, `tests/cases/trial/`.

## Defers

- General semantic classification of arbitrary search text and additional drug sources are outside this fix.
