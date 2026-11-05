# Concurrent first-open cache Busy

Ticket: `0639-cache-concurrent-first-open-busy.md`.

## Cause and fix

Several real processes can fail while opening the same fresh cache. Schema initialization commits before the separate transition to WAL; concurrent openers can each hold a read lock while attempting the exclusive journal-mode upgrade. SQLite may return Busy immediately instead of invoking its busy handler when waiting would deadlock. The pinned libsqlite3-sys 0.38.2 source describes this at `sqlite3.c:3341–3355`; its journal-mode opcode calls `sqlite3BtreeSetVersion`, which starts a read transaction before attempting the write upgrade.

The original native gate j30303 at `7e55a0abc06f32bd12f1f6091d0284288df92c4f` retained only the outer `Fetch::open` Busy. Its exact SQL phase remains unknown. The installed-product witness j30327 independently located a real user-reachable failure at `fetch::cache.rs:125`: the WAL update returned primary code 5 and extended code 5. There was no fixture writer or live database observer.

The fix retries only Busy from the journal-mode query or update. Each failed statement is finalized before waiting, releasing its attempted upgrade so another opener can finish. Every attempt shares one existing five-second WAL wait through `Deadline` and `sql_deadline`; the wait is not enlarged. Each attempt rereads the mode, so it also converges when another opener already enabled WAL. Other errors still return directly. Ownership and schema validation remain before journal changes.

Moving the explicit busy timeout ahead of ownership validation would not fix this: pinned rusqlite 0.40.2 already sets five seconds when opening a connection (`inner_connection.rs:118`). Later zero-timeout discretionary reads and request deadlines cannot cause a failure before `Fetch::open` returns.

## Installed red witness

Job j30327 installed the actual frozen `015aea0a704b7b6176b6988bc6fae707ac0feb38` Linux AMD wheel from independent release custody. Wheel SHA-256: `f7511ad9a4f819dbf248ebd1228e4a73ab2807ebfae075d47b95dfb25d7d7797`. Installed binary SHA-256: `ee51cf73879b228e2b4814056e682ac4d870fa9222628c0b6cfc7f974f740ca9`.

Eight barrier-dispatched `health --offline` commands shared one fresh cache per round, with isolated configuration disabling source groups. The campaign stopped at its first failed round: two rounds passed, and round 2 worker 5 exited 5 with `local_error=busy`. Actual total: 24 commands, 23 passed and one Busy. The failed command took 33.420 ms; one-minute host load was 0.080. The cache had a valid schema and WAL after all processes exited. This disproves fixture-only attribution for the installed failure.

Raw red custody is on Yellow under `data/biomcp-redux-evidence/0639/41e6870a789849096dd6196b21b009c9193c5607/j30327/witness.tar.gz`, SHA-256 `9e32e14d717fb5f6cb008f93c45b6a366ee68ff4c9afefcdac872b374a357acc`.

## Fixed proof

Job j30347 built the official release-profile native Linux wheel from exact source `b0b5206e0674f57c5d065fffc2ff8815808fd6ca`, installed it offline, and used the lasting `release.smoke.cache_first_opens` helper for fifty fresh-cache rounds of eight processes. This is a focused native artifact proof, not full release qualification or a new manylinux claim.

Fixed wheel SHA-256: `48194690a2ca49e5c5346153e246e897ff0767687a959408c5a999a422995d72`. Fixed installed binary SHA-256: `8123fcf1a89e0bc17c0df4f6c43af7d16a8a5d2e7e120119992585661c2d1528`. The wheel's embedded producer record, external manifest and installed bytes agree on source and binary identity.

All 400 commands exited 0 with complete envelopes and empty stderr. Every round began with an empty shared cache and ended in WAL. Five WAL updates actually returned SQLite primary/extended code 5/5 at fixed `cache.rs:337`, then recovered successfully inside the existing bound. Thus the proof exercises the repaired mechanism, rather than merely failing to reproduce it. Dispatch-to-exit durations were 25.570–66.044 ms; one-minute host load was 3.649–4.078. Individual successful schema/WAL phase timings are unobservable in these unmodified binaries; only dispatch duration, error coordinates and post-exit database state are claimed.

Raw green custody is on Yellow under `data/biomcp-redux-evidence/0639/b0b5206e0674f57c5d065fffc2ff8815808fd6ca/j30347/witness.tar.gz`, SHA-256 `7417576f0615680f135222b0f99814cee681c1410058d3c89642a2926f630307`. It contains the artifact, all raw command and numeric diagnostic files, split scanner reports and source-file bindings; the job and partial-cleanup logs are retained beside it.

The lasting installed smoke check runs one eight-process fresh-cache round for each installed channel. The focused fifty-round runner reuses that helper. Existing process-test assertions and timeouts stay intact.

Job j30338 passed canonical cheap checks, all 46 structural ratchets, pinned Ruff and contextual Vulture, five affected Python tests (one skipped), all 17 fetch library tests, and all 28 cache tests, including concurrent processes and ownership-before-journal validation. These results apply only to their unchanged source blobs. Job j30347 completed the remaining pinned duplicate scans: no new Rust product/test or Python tooling clones; the existing Python test-fixture clone is unchanged.

## Limits and setup history

Job j30315's six process-fixture passes did not reproduce the failure and established no root cause. Its timing and empty diagnostic sinks remain retained separately.

Job j30335 stopped at the shell check because a partial precheck goal omitted canonical `make check-cheap`. Job j30336 passed the canonical gate but stopped when a changed-file-only Vulture invocation omitted public `smoke()`'s actual caller in `release/gate.py`. The failed belief was that changed-file-only checks retained entrypoint context; the actual caller disproved it. The canonical goal and caller context repaired these recipes without suppressing a finding or removing code.

Job j30338 remains red: after its green checks, the existing broad debug installed-cleanup fixture exceeded its 300-second parent bound, followed by a secondary temporary-directory assertion. Captured output cannot identify the exact stalled child stage. This was not a witnessed WAL Busy, and its timeout was not enlarged. Owned process/temp cleanup was verified before reuse. The coordinator scoped the remaining-only continuation to scans, the fixed release wheel and the installed startup campaign; no green checks or broad cleanup rehearsal were repeated. Final candidate qualification must cover the full installed smoke separately.

Earlier cases have distinct established causes: 0296 fixed mixed ownership snapshots; 0287 added typed local errors while leaving startup contention unproven; 0569 addressed a request deadline without changing this startup path. Their shared diagnostic gap was retaining outer errors without operation coordinates and timing. The existing numeric SQLite sink plus retained process/installed witnesses now distinguish that gap from the specific WAL startup defect fixed here.
