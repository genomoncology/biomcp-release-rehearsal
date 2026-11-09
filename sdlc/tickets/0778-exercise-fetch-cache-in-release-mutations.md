# Exercise the owning cache API in release mutations

Status: COMPLETE.
Landed: 57b7a89f53c5192e79a7671d03aa26e0c273b96a

Milestone: 5

## Outcome

Release mutations exercise the existing public Fetch cache integration target alongside CLI cases, and detect changes to retention, corrupt-row handling, removal and filesystem error classification.

## Why

The release runner omits the owning API tests, allowing meaningful cache changes to survive its CLI-only test selection.

## Proof

- Proof: the existing cache integration target verifies expired rows on both sides of the thirty-day retention boundary, corrupt cached status/pagination, removal and a Unix missing-parent filesystem error. The release runner invokes this target and CLI cases together. Focused mutation witnesses from j36291 are caught; the only excluded mutation is the proven-equivalent disjoint SQLite flags OR-to-XOR replacement, with an exact documented match applied consistently to inventory and shards. Use the existing configured checks; the build delegate owns the bounded final mutation run after landing.

## Starts from

- Starts from: `release/mutations.py::native_args`, `release/test_mutations.py`, `crates/biomcp-fetch/tests/cache/{storage,freshness}.rs`, and `crates/biomcp-fetch/src/cache.rs`. Raw j36291 evidence is retained on Yellow under `data/biomcp-redux-evidence/0640/42a79b1bbdc9/`; the fresh findings reviewer has actual survivor diffs.

## Keeps

- Keeps: runtime cache behavior, schemas, ownership/WAL deadline, source/API behavior, mutation budget and truthful caught/survivor/unrun reporting. Reuse existing public API fixtures; do not duplicate coverage in a new CLI harness or change runtime code to kill an equivalent mutation.

## Defers

- Defers: unrelated test gaps and broader mutation selection changes.

## Changes

- Changes: `release/mutations.py`, `release/test_mutations.py`, the existing Fetch cache integration tests, and a narrowly scoped existing cargo-mutants exclusion setting if needed.
