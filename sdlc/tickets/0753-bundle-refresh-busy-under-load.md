# 0753 — Bundle refresh stays out of busy under load

Status: READY.

Milestone: 5

## Outcome

`cases/gene-sections/bundle-modified-refresh` passes under the load test, or the wait that returned Busy is shown to be a correct bounded wait and the case allows for it. Then the checks the load test has not yet reached run once.

## Why

The load test j34423 at d0f18a16c, with the first-open lock fix in, failed this case on its first round with `local_error: busy`, exit 5, at host load about 10. No SQLite step was recorded, so the Busy came from a wait other than the open steps. 0641 says one failure opens a ticket. 16 of the 30 checks in that run stayed unreached.

## Starts from

- Starts from: `crates/biomcp/tests/runner/bundle_sync.rs`, `crates/biomcp-fetch/src/bundle/store.rs`, and which wait can return Busy with no SQLite step.

## Keeps

- Keeps: the store format, namespace and BioData pin. A timing test is fixed or deleted, not rerun.

## Changes

- Changes: name the wait first. Fix the cause if it is in the product; fix the case if the wait is right.

## Proof

- Proof: `make stress STRESS_ARGS="--only cases/gene-sections/bundle-modified-refresh"` passes on the Linux host, then the unreached checks run once.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: j34423: 12 checks passed, 112 runs, first-open passed all 50 rounds; then this case failed.

## Progress
