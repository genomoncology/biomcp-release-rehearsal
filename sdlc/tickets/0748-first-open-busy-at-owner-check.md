# 0748 — First open waits out busy at the owner check

Status: READY.

Milestone: 5

## Outcome

Concurrent first opens of a fresh cache never return Busy at the owner check. Each process waits within the existing five-second open limit and then succeeds.

## Why

Under load, 7 of 8 concurrent first opens exited 5 with `local_error: busy` in round 28 of the `first-open` check of `make stress` (j34269, commit bf5f97838). All seven failed at `fetch::schema.rs:53`, the read of `sqlite_master` in `validate_owners`, with SQLite code 5. 0639 made only the journal-mode switch retry. This read comes after it and has no retry. The cache code at that commit matches main. The 50-round run on a quieter host (j34194) did not see it. 0641 says one failure opens a ticket.

## Starts from

- Starts from: `crates/biomcp-fetch/src/cache/schema.rs` and `enable_wal` in the cache open path.
- First find out whether the Busy came at once or after the full wait. A wait that ran out is a different cause from a refusal that skips the busy handler.

## Keeps

- Keeps: the five-second open limit, shared across every step of the open, the store format, the cache namespace and the schema version.
- Keeps: a caller's open transaction, as `check_owners` says.

## Changes

- Changes: the owner check retries Busy under the same deadline, as `enable_wal` does, with its read snapshot closed before each wait. Apply it at any other step of the open with the same gap.

## Proof

- Proof: a test that holds the failing state fails before the fix and passes after it. The `first-open` check passes under load on Linux, on the job that closes 0641 and 0670.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: j34269 log, round 28: `exits [5, 0, 5, 5, 5, 5, 5, 5]`, host load about 9.6, 2 pinned cores, 3 burners.

## Progress
