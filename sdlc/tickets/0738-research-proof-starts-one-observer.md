# 0738 — Research proof starts one observer

Status: OPEN.

Milestone: 5

## Outcome

`live-source-checks/report-is-one-json-value` passes on every Unix platform, so the practice release gets past its install step. The public scripts hold no local workspace paths.

## Why

Practice release run 42314814887 stopped on Linux Intel, Linux arm64, Mac Intel and Mac arm64 at this one proof. 0681 changed `call_checked` in `crates/biomcp/tests/runner/research.rs` to start a separate server to list tools with the same `--observe` file, and that file can only be created once (`crates/biomcp-fetch/src/observe.rs:182`). Windows skips this proof and passed. Routine checks don't run proofs, so it first ran in the release gate. `scripts/deadline_setup_load.py` also holds a local path.

## Starts from

- Starts from: `crates/biomcp/tests/runner/research.rs`, `scripts/deadline_setup_load.py`.

## Keeps

- Keeps: `crates/biomcp-fetch/src/observe.rs` as it is; only the test helper is wrong. No new checking tool.

## Changes

- Changes: `crates/biomcp/tests/runner/`, `scripts/`.

## Proof

- Proof: the job that reproduced it in 7.5 seconds (j33789's command) passes. Every other proof that the release gate runs and routine checks skip also passes on one job, so the practice release doesn't find the next one.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.
