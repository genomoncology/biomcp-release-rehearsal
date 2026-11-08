# 0770 — First snapshot wait case is not timed

Status: OPEN.

Milestone: 5

## Outcome

`cache-resilience/hgnc-first-snapshot-wait-is-bounded` gives the same result on every host and load, and a gene read that stops because the HGNC bundle is busy or out of time always names why.

## Why

Practice release run 42451038807 at dee83eff6 failed in the macOS arm64 `qualify` job, installed cases: "first snapshot did not wait within its deadline". The HGNC source came back `state: Failed, reason: None` with `local: Some(Cache(Busy))`, where the case expects reason `Syncing` or `DeadlineExceeded`. The case passed on Linux in the same run, on the M5, and in every earlier practice run. It gives the read a 30 ms budget (`crates/biomcp/tests/runner/hgnc_sync.rs:316`), so on a slow host the budget runs out at a point other than the bundle read's own check. A timing test is fixed or deleted, not rerun.

Two readings, and the developer settles which: the product drops the reason when the deadline or a busy cache stops the read at another step (then a failed source with no reason is a product bug and is fixed), or the case's wall-clock budget is the only problem (then the case drives the same state without depending on time).

## Proof

- Proof: the case reaches the busy-first-snapshot state without a wall-clock race and asserts the reason. If the product could return `Failed` with no reason here, a case that reaches that path deterministically fails on main today and passes after the fix.

## Starts from

- Starts from: `crates/biomcp/tests/runner/hgnc_sync.rs` `bounded` (lines 292-336) and the HGNC bundle read and deadline checks in `biomcp-query` that set `Reason::Syncing` and `Reason::DeadlineExceeded`.

## Keeps

- Keeps: the store format, cache namespace and BioData pin; today's reasons and their names; `hgnc-first-snapshot-recovers`.

## Defers

- Defers: nothing.

## Changes

- Changes: that case, and the query code only if it drops a reason.

## Depends on

- Depends on: nothing.
