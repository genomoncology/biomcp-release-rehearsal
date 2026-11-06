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

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Before editing, list the real rows to remove: paper IDs and DOIs from the Semantic Scholar replies, and product references from the WHO capture. The Proof greps for that list, because a search for "no real row" with no list cannot fail.

`tests/cases/drug/who-dates.yaml` asserts 657 rows and holds real product references, so the case is rewritten in the same commit as its capture, and `scripts/check_who_dates.sh` is rechecked. Replies are keyed by request hash, so editing a body keeps the key. Semantic Scholar replies that hold only 429 error bodies have no rows and stay. The Semantic Scholar part follows 0717, which changes three of those replies.
