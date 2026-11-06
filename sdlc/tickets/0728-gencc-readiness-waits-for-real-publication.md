# 0728 — GenCC readiness waits for real publication

Status: COMPLETE.
Landed: 7f776a177844873fc793160357f3b86af23eb3f4

Milestone: 5

## Outcome

The GenCC first-publisher case waits on the actual readiness event, its short reader deadline starts after publisher admission, and a typed Busy is accepted only with exit 5.

## Why

Split from 0641 (m28818) so this already accepted repair can land in the batch without closing 0641's repeated-load work and row W24. It repairs j31824's failure.

## Proof

- Proof: focused ten-load rounds j32394 and routine j32395 on reviewed source d980dadaaf.

## Starts from

- Starts from: 0641, and the case in `gene-sections/gencc-first-publisher-deadline`.

## Keeps

- Keeps: 0641 open for W24 and the full repeated-load campaign. The ordinary case still needs real publication data.

## Defers

- Defers: nothing.

## Changes

- Changes: that test and its helpers.
