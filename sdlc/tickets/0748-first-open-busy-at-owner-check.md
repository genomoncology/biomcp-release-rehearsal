# 0748 — First open waits out busy at the owner check

Status: COMPLETE.
Landed: 1162eae8a24ecf46869e76da04bb3bcd8a8ad6ff

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
- Keeps: the `open.<step>` failure note and the first-open script's duration print.

## Changes

- Changes: `open_with_owners` serializes setup across processes with an OS file lock on a sibling `open-lock` file, polled under the same five-second deadline and then Busy. It is held only for the owner check, initialize and WAL switch. Every open takes it, with no unlocked shortcut, because that is simpler. A file system without locks proceeds unlocked. The one new file is the lock file. The Busy retry loops are removed.
- Changes: the `quotas` spacing bounds allow arrival jitter (150 ms, and 750/1500 ms).

## Proof

- Proof: a test that holds the failing state fails before the fix and passes after it. The `first-open` check passes under load on Linux, on the job that closes 0641 and 0670.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: j34269 log, round 28: `exits [5, 0, 5, 5, 5, 5, 5, 5]`, host load about 9.6, 2 pinned cores, 3 burners.

- Finding: j34329 (2f165897d), round 13: seven openers failed at `open.owners` after 5005 to 5014 ms with 0 retries, so one read sat in SQLite's busy handler for the whole five seconds. The eighth won at 5531 ms, about 100 ms after the others quit at about 5430 ms. Waiting openers appear to hold up the one doing setup. The mechanism inside SQLite is unproven.

## Progress
