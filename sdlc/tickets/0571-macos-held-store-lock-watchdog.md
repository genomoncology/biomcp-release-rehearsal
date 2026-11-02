# macOS ARM held store lock watchdog

Status: OPEN.
Milestone: 5

## Outcome

`pathway-wikipathways/held-store-lock-names-local-failure` passes on installed macOS ARM: while another connection holds a write lock on the replay cache, each MCP call returns within its bound, the reader still answers from the fetched catalog, and the sync reports a named local failure.

- Why: private rehearsal run 41376870296 at 0fea901b passed the candidate, then macOS ARM qualification failed 1 of 527 installed cases with only "publication watchdog" (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41376870296/macos-arm-job.log` lines 1665-1672). The watchdog fires when the server sends no reply for 30 seconds (`crates/biomcp/tests/runner/publication.rs:61-65`). The case holds `BEGIN IMMEDIATE` on `case-cache/replay.sqlite` and then calls search and sync through MCP (`crates/biomcp/tests/runner/cache_contention/store_locks.rs:22-83`). Native M5 job j25774 passed all 527. A server that stays silent for 30 seconds behind a held lock would hang a real user's agent, so the cause matters.
- Starts from: the two files above, the product's SQLite busy handling for the cache and store, and the raw log.
- Rules: bounded diagnosis first: name which call went silent (search or the second sync) and where the server waited, from the log or one focused installed reproduction on a Mac. If the product waits past its own busy bound, fix the product. If the proof's 30-second watchdog is shorter than a documented product bound, say which bound wins and why. Do not raise a deadline without that reason, and run no extra Actions run.
- Proof: a reproduction that fails before and passes after, or a measured explanation, and the next private rehearsal on macOS ARM.
- Changes: the product's busy path or the proof, whichever the diagnosis names.
- Defers: other rehearsal failures, each to its own ticket.
