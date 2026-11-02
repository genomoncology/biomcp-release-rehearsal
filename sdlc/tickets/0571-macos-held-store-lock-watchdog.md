# macOS ARM held store lock watchdog

Status: COMPLETE.
Landed: 85707b100df30ed6b196b16e8c730ba3899124dd
Milestone: 5

## Outcome

`pathway-wikipathways/held-store-lock-names-local-failure` passes on installed macOS ARM: while another connection holds a write lock on the replay cache, each MCP call returns within its bound, the reader still answers from the fetched catalog, and the sync reports a named local failure.

- Why: private rehearsal run 41376870296 at 0fea901b passed the candidate, then macOS ARM qualification failed 1 of 527 installed cases with only "publication watchdog" (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41376870296/macos-arm-job.log` lines 1665-1672). `Peer::next` (`crates/biomcp/tests/runner/publication.rs:61-65`) reports "publication watchdog" both when no reply comes within 30 seconds and when the server's output closes, so the log shows neither 30 seconds of silence nor which phase failed (m19830). macOS Intel failed the same case (`macos-intel-job.log` lines 1960-1969). The case holds `BEGIN IMMEDIATE` on `case-cache/replay.sqlite` and then calls search and sync through MCP (`crates/biomcp/tests/runner/cache_contention/store_locks.rs:22-83`). Native M5 job j25774 passed all 527. A server that stalls or exits behind a held lock would hurt a real user's agent, so the cause matters.
- Starts from: the two files above, the product's SQLite busy handling for the cache and store, and the raw log.
- Rules: bounded diagnosis first: name which of the three MCP calls failed (the initial sync before the lock, the search under the lock, or the second sync under the lock), tell a timeout from a closed channel, keep the child's status and stderr, and find where the server waited, from the log or one focused installed reproduction on a Mac. If the product waits past its own busy bound, fix the product. If the proof's 30-second watchdog is shorter than a documented product bound, say which bound wins and why. Do not raise a deadline without that reason, and run no extra Actions run.
- Proof: a reproduction that fails before and passes after, or a measured explanation, and the next private rehearsal on macOS ARM.
- Changes: the product's busy path or the proof, whichever the diagnosis names.
- Defers: other rehearsal failures, each to its own ticket.
