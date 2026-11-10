# 0793 — Preserve uncertainty in GWAS threshold totals

Status: OPEN.

Milestone: 5

## Outcome

A GWAS threshold search never reports an exact qualifying total when an observed association cannot be compared with the threshold or an association was refused. It retains comparable results with an honest incomplete status and diagnostic.

## Why

The November 10 review found that a missing exact p-value is skipped while the walk can still return complete and a precise qualifying total. Evidence: `records/2026-11-10-product-review.md`, F1.

## Starts from

- `crates/biomcp-query/src/gwas/threshold.rs` (`append`, `render`), `crates/biomcp-adapters/src/gwas/receive.rs`, the native page refusal representation, and the existing GWAS threshold cases. Missing p-values do not currently set `Saved.dropped`; `qualifying_total` depends only on `Saved.done`.

- PM decision, 2026-11-10: keep usable comparable rows, classify the incomplete observation through the existing shared status owner, and omit `qualifying_total`. Do not substitute zero or label a lower bound exact. Carry uncertainty through continuations and saved replies. Conservatively suppress the exact total for an unresolved native refusal; distinguish harmless child-field omissions only if the existing representation can prove they leave association membership known.

## Proof

- Routine outside-in replay: valid qualifying association plus missing/invalid p-value; a refused association; uncertainty on an earlier page followed by an ordinary terminal page; and rereading the retained pool. Each preserves the problem and never claims complete exact membership. An all-incomparable terminal page is not Complete and has no exact qualifying total even when zero rows are returned. Controls cover valid zero matches, equality, underflow and a fully comparable terminal page with its correct exact total. Follow the existing case runner; no live GWAS fetch is needed.

## Keeps

- Exact decimal comparison, native order, existing bounded page walk, source total meaning, source attribution and current refusal behavior for invalid ordering/duplicate pages.

## Changes

- Changes: `crates/biomcp-query/src/gwas/threshold.rs`, `crates/biomcp-adapters/src/gwas/`, `tests/cases/gwas/`.

## Defers

- Shared pool reuse is owned by the source-snapshot ticket; coordinate its eligibility input without broadening this fix.
