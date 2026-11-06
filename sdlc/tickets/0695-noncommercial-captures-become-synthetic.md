# 0695 — Noncommercial captures become synthetic

Status: OPEN.

Milestone: 5

## Outcome

The WHO prequalification export and the roughly 21 real Semantic Scholar replies in the repo are replaced by synthetic rows with the same shape, and the cases still pass. History is not rewritten.

## Why

Row W11 of `records/2026-11-06-pre-release-review.md`. Both sources are labeled `noncommercial`, and the tree goes to an MIT public repo at the append. Rule: `AGENTS.md:5` and 0476's precedent; public repo history stays (`instructions/decisions/2026-10-19-public-repos-skip-history-scrubs.md`). The PM notes this in Ian's approval packet.

## Proof

- Proof: every case that used a capture passes on synthetic rows, and a grep shows no real row remains. No new check until this happens a third time. Routine gate.

## Starts from

- Starts from: `tests/replies/` for both sources.

## Keeps

- Keeps: each case's meaning. No real records and no live fetch of noncommercial data for this.

## Defers

- Defers: nothing.

## Changes

- Changes: `tests/replies/`, the cases that read them.
