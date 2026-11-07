# 0744 — DOI fallback row keeps a test

Status: OPEN.

Milestone: 5

## Outcome

The DOI fallback row in `observe_context.rs` has a test again.

## Why

0732 deleted `live_witness_proof.rs`, which held the only test of that row.

## Starts from

- Starts from: `observe_context.rs` and the deleted proof at its last commit.

## Keeps

- Keeps: recorded cases, with no new live call.

## Changes

- Changes: one case or test beside the code.

## Proof

- Proof: the test fails with the row removed.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.
