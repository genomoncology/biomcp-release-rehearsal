# 0641 — Multi-process cache checks run under repeated load

Status: OPEN.

Milestone: 5

Depends on: 0293

## Outcome

One Make goal runs the multi-process cache checks many times on a loaded machine against a current main build, offline. Any single failure ends the job as failed and names the SQLite operation and code, so a race that fails one run in twenty-four is caught within days instead of at release.

- Why: `records/2026-11-05-release-finds-one-at-a-time.md`. The 0639 startup race passed routine checks for weeks because they run `cache_process::concurrent_first_opens_and_distinct_writes_survive_into_a_new_process` once. The same test had failed under load before (0569), and its failing SQLite step was never recorded.
- Starts from: 0639's fresh-cache campaign (`release.smoke.cache_first_opens` and `scripts/check_fixed_cache_first_open.sh`), 0569's `scripts/check_cache_process_load.sh`, and the `nightly-live-check` schedule in pm's `config/jobs.json`.
- Rules: reuse those runners, with no new framework. Round counts and load are named constants; the default matches 0639's proof of fifty rounds of eight processes. A failure is never retried or marked flaky; it opens a ticket. No source keys and no network. The schedule entry lives in the pm repository, so the coordinator asks pm's owner to add it beside `nightly-live-check`.
- Proof: the goal passes at main and records its rounds. The same goal against the retained 015aea0a7 wheel, which predates 0639's retry, fails with Busy at the `fetch::cache.rs` WAL switch.
- Changes: `scripts/check_cache_process_load.sh`, `Makefile`; the stress goal.
- Defers: other multi-process tests, until one of them fails under load.
