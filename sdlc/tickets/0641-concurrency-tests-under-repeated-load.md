# 0641 — Real-time and multi-process tests run under repeated load

Status: COMPLETE.
Landed: 8f0beb2c93fd598cb5c84418a89e894996c14572

Milestone: 5

## Outcome

One Make goal runs the multi-process cache checks, and every test that uses real time or child processes, many times on a loaded machine against a current main build, offline. Real-time tests are the `release/` and `scripts/` Python test files in `docs-check`, the live-stub tests in `biomcp-fetch` and the runner's live trials, listed by name. Load is about 1.5 times the cores. It runs nightly, and on demand for touched files (about 10 runs) when a ticket adds or changes such a test. Any single failure ends the job as failed and names the SQLite operation and code, so a race that fails one run in twenty-four is caught within days instead of at release.

- Why: `records/2026-11-05-release-finds-one-at-a-time.md` and `records/2026-11-05-timing-tests-under-load.md`. The 0639 startup race passed routine checks for weeks because they run `cache_process::concurrent_first_opens_and_distinct_writes_survive_into_a_new_process` once. The same test had failed under load before (0569), and its failing SQLite step was never recorded.
- Starts from: 0639's fresh-cache campaign (`release.smoke.cache_first_opens` and `scripts/check_fixed_cache_first_open.sh`), 0569's `scripts/check_cache_process_load.sh`, and the `nightly-live-check` schedule in pm's `config/jobs.json`.
- Rules: reuse those runners, with no new framework. Round counts and load are named constants; the default matches 0639's proof of fifty rounds of eight processes. A failure is never retried or marked flaky; it opens a ticket. No source keys and no network. The schedule entry lives in the pm repository, so the coordinator asks pm's owner to add it beside `nightly-live-check`.
- Proof: the goal passes at main and records its rounds. The same goal against the retained 015aea0a7 wheel, which predates 0639's retry, fails with Busy at the `fetch::cache.rs` WAL switch.
- Changes: `scripts/check_cache_process_load.sh`, `scripts/cache_process_load.py`, `Makefile`; the stress goal and its test list.
- Defers: nothing. The earlier clause deferring other tests until one failed under load was itself a cause of finding them one gate failure at a time (`records/2026-11-05-timing-tests-under-load.md`).

## Amendment, 2026-11-06

Runs before the final rehearsal (W28 of `records/2026-11-06-pre-release-review.md`). Also owns W24: measure whether a large cache write stalls reads and `/health`. Include the `gencc-first-publisher-deadline` failure of j31824.

From the ticket review, `records/2026-11-06-ticket-review.md`:

The branch's four commits moved to 0728 and landed. Nothing else is on it, so release the lane and restart from main.

The work splits into three parts, in order: one Make goal with the named test list, the W24 measurement, and the nightly entry. Load is 1.5 times the job's booked cores, not the machine's cores, because builders are shared. The wheel proof names where its wheel comes from before it runs.

Under Ian's 2026-11-06 ruling, these tests go in the long suite at final release, not on a schedule. The nightly entry is dropped. A test that fails on timing is fixed or deleted at once.
