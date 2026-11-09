# 0787 — include-owning-regressions-in-release-mutations

Status: COMPLETE.
Landed: c582d2b35cb9edfddaab021ec5ec6a2e1fd1c889

Milestone: 5

## Outcome

The existing bounded mutation runner executes the owning public Fetch bundle/overflow and query health regression targets added by the final correctness fixes, alongside the existing CLI cases and Fetch cache tests. Its native baseline visibly runs the selected union.

## Why

The explicit test allowlist fixed for cache coverage in 0778 omits the newly relevant integration targets. A bounded mutation result must not silently omit the tests that prove the changed behavior.

## Proof

- Extend the existing mutation-helper command test to verify the exact package and target union for discovery and shards; run that focused helper suite. Do not run a mutation campaign before landing.
- During final selected-source qualification, retain the actual native baseline and selected/completed/unrun counts. Keep old calibration labelled with its original test matrix; use a new measurement or an explicitly conservative estimate for the wider matrix.

## Starts from

- `release/mutations.py::native_args`, `release/test_mutations.py`, 0778 and the owning regressions in 0781/0782/0784. Add the query package plus Fetch bundle, transport, double_gzip, xml, record and query health targets. The CLI configuration trial already runs through cases; do not add the separate tools runner.

## Keeps

- Existing runner, 90-minute cap, deterministic bounded selection, baseline execution, exact equivalent-mutant exclusion, source/plan binding and honest incomplete/unrun reporting. No new test runner, selection framework, broad workspace campaign or product behavior change.

## Defers

- Actual bounded mutation execution belongs to final selected-source qualification after the product batch lands.

## Changes

- The existing mutation helper target list and its existing command-contract test.
