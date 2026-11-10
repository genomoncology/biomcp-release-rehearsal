# 0796 — Advance trial pages by native record count

Status: COMPLETE.
Landed: 5acd61b635a81d79fe7e68342208d2dd187ab84d

Milestone: 5

## Outcome

ClinicalTrials.gov continuations advance their offset and end-of-results calculation by the number of native records consumed, even when malformed records are refused. Returned counts still describe admitted records.

## Why

The opaque provider token advances past every native row, but the query currently advances its offset by the shorter admitted list. Evidence: `records/2026-11-10-product-review.md`, F4.

## Starts from

- `crates/biomcp-adapters/src/ctgov/mod.rs` (`SearchPage`, native mapping) and `crates/biomcp-query/src/trial.rs` (`apply_page`), plus existing recorded trial continuation cases. Count native slots before filtering invalid records.

## Proof

- Routine installed replay: two native pages of two records, one refused record on the first page, known total four and the provider's full-final-page token. Assert next offset two, admitted counts one then two, partial first response with its diagnostic, and no third empty page request. Include an all-refused first page, unknown total and ordinary complete-page controls.

## Keeps

- Opaque source tokens and source order, current filters/name expansion, count-only behavior, exact native total and honest partial status. This fixes offset/end detection; no evidence establishes skipped or duplicated accepted records.

## Changes

- Changes: `crates/biomcp-adapters/src/ctgov/mod.rs`, `crates/biomcp-query/src/trial.rs`, `tests/cases/trial/`.

## Defers

- No generic pagination rewrite; use the native received-count pattern already used by other adapters.
