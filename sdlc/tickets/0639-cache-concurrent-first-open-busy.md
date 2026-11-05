# Concurrent first opens of a fresh cache never fail as busy

Status: OPEN.
Milestone: 5

## Outcome

Several BioMCP processes opening the same fresh cache at once all succeed, or wait within the command budget, instead of one failing with `Cache(Busy)`; the root cause is named and fixed where it lives.

- Why: native gate j30303 at candidate 7e55a0ab (2026-11-05) failed `cache_process::concurrent_first_opens_and_distinct_writes_survive_into_a_new_process`: a child's `Fetch::open` returned `Cache(Busy)` at `crates/biomcp-fetch/tests/support/cache_process.rs:226` (m25998). Idea 0538 was declined on 2026-11-01 with the condition "If a first-open SQLITE_BUSY failure is ever observed"; it now has been. Cache concurrency failures recurred before (0287, and 0060's count checks). Agents commonly run several CLI processes in parallel on a fresh machine, so a real user may hit this.
- Starts from: a short root cause analysis (`skills/rca.md`) of `Cache::open_with_owners` and the open path in `crates/biomcp-fetch/src/cache.rs` (the `busy_timeout(OPEN_WAIT)` open, the zero-timeout path near line 271, the deadline-scaled timeout near line 343), `crates/biomcp-fetch/src/cache/schema.rs` (Busy mapping), idea 0538's reasoning, and the failing test and its witness.
- Confirmed 2026-11-05: installed 015 binary, 8 processes on a fresh cache, round 2 worker 5 exited busy at `crates/biomcp-fetch/src/cache.rs:125` (the `journal_mode=WAL` pragma), j30327; the earlier focused rerun j30315 did not reproduce. RCA: `records/2026-11-05-cache-first-open-busy.md`.
- Rules: first decide by evidence whether real processes can hit this or only the test's timing; record that in the RCA. Fix the general cause (for example a schema-creation or owner-check step that runs with a zero busy timeout or outside a wait), not the test. No longer timeouts as the fix unless the RCA shows the wait is the right behaviour and it stays within the command budget. If the RCA shows a user-reachable defect, tell the PM before landing, because it reopens the frozen 1.0 candidate.
- Proof: the failing test passes repeatedly under load (the RCA states the count and load), plus one installed-binary case starting several processes on an empty cache at once; existing cache tests unchanged.
- Changes: `crates/biomcp-fetch/src/cache.rs`, `crates/biomcp-fetch/src/cache/**`, `crates/biomcp-fetch/tests/support/cache_process.rs`, `sdlc/records/`; the RCA, the fix and its proof.
- Defers: nothing.
