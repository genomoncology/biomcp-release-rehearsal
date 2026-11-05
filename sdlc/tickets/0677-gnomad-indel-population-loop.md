# 0677 — Variant population answers indels

Status: OPEN.

Milestone: 5

## Outcome

`variant population` answers an indel that gnomAD has, given as chrom-pos-ref-alt or as the HGVS form `get variant` prints, and never advises a command that refuses the same variant. Where a form truly cannot be answered, the advice names one that can.

## Why

The bench recheck of the 1.0 candidate (Jarvis m27105, report `experiments/513-biomcp-cli-codesign/climb/gaps-on-rc-ed9c36a96.md` in the bench repo): `biomcp variant population 7-117559590-ATCT-A` exits 4 and advises `get variant`, whose HGVS form `variant population` refuses with the same advice. gnomAD has the variant.

## Proof

- Proof: Investigate first: reproduce on current main in single and typed mode; if main already answers, record the evidence and close. Otherwise replayed cases for a deletion and an insertion in both input forms, and a structural check that no refusal's advice names a command that refuses the same input. Fix the general normalization cause; tune nothing to the example. Routine gate.

## Starts from

- Starts from: the variant population command and gnomAD adapter in `crates/biomcp-query/src/variant/` and `crates/biomcp-adapters/`, and the variant normalization shared with `get variant`.

## Keeps

- Keeps: SNV behavior and existing population results.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/variant/`, the gnomAD adapter if needed, `tests/`.
