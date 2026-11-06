# 0724 — Source text arrives clean

Status: OPEN.

Milestone: 5

## Outcome

Replies carry readable text: HTML entities decoded, damaged characters such as `?` for `≥` repaired at their source or marked, markdown escapes removed from trial results, FAERS codes shown as words beside the code, and GWAS p-values below the float range shown from the exact value, not 0.0.

## Why

Item 8: EMA `&nbsp;` and `?` for `≥`; trial results `\<` and `\^2`; FAERS outcome `6`; GWAS 0.0 below about 1e-300. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then recorded-reply cases for each. Check first whether EMA's `?` is in the source bytes or made by our decode; if it is in the source, say so and leave the value. One shared text cleaner, not one per adapter. Raw values stay available where they are identifiers.

## Starts from

- Starts from: `crates/biomcp-adapters/src/faers/detail_map.rs`, EMA, ClinicalTrials.gov and GWAS adapters.

## Keeps

- Keeps: meaning of source text. No guessed characters.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/`, `crates/biomcp-types/`, `tests/`.
