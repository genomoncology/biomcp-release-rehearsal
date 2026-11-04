# 0596 — HGNC sync proofs distinguish ordinary and fresh reads

Status: COMPLETE.
Landed: 7216c50d98475da1e999576a37efc36a44926cee

Milestone: 5

## Outcome

The HGNC live-sync proof lets an ordinary reader use published data without taking the active publisher's lock, proves that an explicit fresh reader waits until the publisher ends, and runs that regression in the existing routine gate.

## Why

The public rehearsal of current main fails on Windows and Linux ARM because the installed-only proof requests fresh data while holding its publisher open, then expects immediate success.

## Proof

- Proof: Rehearsal 41757168961 at source `974d11223696ce12ac4885f92b0c7789beeb9b42` fails `cache-resilience/hgnc-trials-during-live-sync` in Windows job 126336913104 and Linux ARM job 126336913152: expected exit 0, received exit 4 with HGNC `syncing`. Use the retained installed artifact and existing runner for the focused correction. Assert ordinary-reader success with the publisher still held and unchanged. Observe an explicit fresh reader waiting before a controlled publisher release or termination, then require fresh-reader success. Keep the abandoned-publisher recovery proof. The ordinary routine gate must execute the small live-sync regression, rather than only compiling it. Use state or observation barriers, without a slowness-only assertion or fixed sleep.

## Starts from

- Starts from: `crates/biomcp/tests/runner/hgnc_sync.rs` functions `args`, `trial`, `Pending`, `live` and `abandoned`; `args` currently appends `--fresh` to every trial request. The module and its trials are restricted to the tool runner in `crates/biomcp/tests/runner/mod.rs`. `crates/biomcp/Cargo.toml` declares that runner with `test = false`, so the ordinary workspace test and coverage run omit it. `release/smoke.py` runs the installed tool proofs. The channel-controlled pattern in `runner/bundle_refresh.rs` already tests fresh-reader waiting.

## Keeps

- Keeps: Product behavior from 0592 stays unchanged: explicit fresh reads wait, while an ordinary read may use already-published rows during an active publisher. Seed and abandoned-publisher recovery requests still ask for fresh data. Keep installed proof registration exactly once per runner, existing routine commands and fixture isolation. Change tests and their selection only; add no release gate or product mechanism.

## Defers

- Defers: Nothing.

## Changes

- Changes: `crates/biomcp/tests/runner/hgnc_sync.rs`, `crates/biomcp/tests/runner/mod.rs`, and existing shared observation or publisher helpers if needed.
