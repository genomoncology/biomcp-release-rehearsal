# Tests that fail on timing when the machine is busy

Date: 2026-11-05. Read-only analysis of `origin/main` at dea3f1389 (code unchanged at 52a4cb50a), pm job lists and logs, tickets and records. No tests or jobs were run. Each claim below is marked **confirmed** (seen in code, a job log or the job list) or **likely** (inferred).

## Answer

Yes, an analysis was warranted. The same kind of failure hit a second ticket and then a third: batch gate j30742 failed, the first fix covered only the test that failed, and the corrected batch j30773 failed on a sibling test (confirmed). The cause is wider than `release/test_mutations.py`. Four more tests on main make the same assumption and one of them already failed once without being ticketed (j30393, confirmed). Eleven lower-risk tests share the shape.

## Cases

Load is the one-minute load average when pm started the job, as `pm job list` reports it, not the load during the failing test.

| Case | Job | Where it ran | What failed | Cause group |
|---|---|---|---|---|
| 0667, first | j30742 (batch 0663+0665, candidate 11650a9aa) | m5, load 14.7 of 18 cores, beside 5 jobs | `test_version_check_obeys_the_shared_deadline`: a 2 s budget ran out before fake Cargo started, so `cargo-calls.jsonl` never existed (confirmed, log) | A |
| 0667, second | j30773 (candidate abbbfd9d8, which fixed only the version test) | m5, load 19.7 of 18, beside 6 jobs | `test_deadline_preserves_partial_native_outcomes_without_fresh_budget`: a 3 s budget ran out in the planning step's version check ("Install the pinned cargo-mutants before planning") before any shard ran (confirmed, log and diff) | A |
| 0667, third | none, found by reading | | Owned-child test armed 5 s before the nested child was ready (confirmed, removed in dea3f1389) | A |
| Unticketed | j30393 (0642 branch, which did not touch the gate, smoke or test files) | Yellow, load 0.02 of 4 | `release.test_gate ... test_installed_cli_rows_bind_results_and_fail_on_cli_exit`: `tool-runner --proofs` "timed out after 0.0556 seconds" (confirmed, log). No ticket or record mentions it (confirmed, grep) | A |
| 0639 record | j30338 | gator, load 0.2 of 12 | `release.test_smoke ... test_isolated_installed_commands_and_cleanup` went past its 300 s limit. The section later passed in 258 s (j30464, 0650). The stalled stage was never identified (confirmed, record) | A (likely) |
| 0103 | j10684 | from ticket | HTTP slow-header test timed the server's close event together with test cleanup. Its isolated rerun passed (ticket text) | A |
| 0286 | review finding | | HTTP close-time test had an upper bound that failed under coverage and parallel load (ticket text) | A |
| 0622 | j29497 (cold matrix, 99 min) | m5 under x86 emulation, load 1.8 | A bare socket `read` returned `Interrupted` in two sub-checks of `mcp-http/framing-resource-deadlines-and-shutdown` (confirmed, log lines for slow header and rejected framing). Fixed for every runner read through one helper | B |
| 0569 | j25631 | Linux ARM64 container | Cache-process child got `DeadlineExceeded` inside its 10 s budget. The fix moved SQLite fixtures off Docker Desktop host mounts, which "lost committed rows" (confirmed, `scripts/linux_fixture_cache.sh` comment) | C (environment) |
| 0639 | j30303, j30327 | m5, gator | Same cache test. It was a real startup race in the product (Busy at the WAL switch) | C (product bug) |
| 0287 | j14991, j16003, j16035 | from ticket | Opaque concurrency failures that passed on rerun. Typed errors were added, and startup contention was later proven by 0639 | C |
| 0255, 0272, 0351 | j14991, j15216, j18030 | from tickets | Failed once and passed on rerun. Closed with extra diagnostics and no cause found | D |

Count: 13 cases. **Group A** (a test assumes work finishes inside a real-time budget) has 7, of which 4 broke a gate or a release section. **Group B** (a test assumes the OS never interrupts a call) has 1. **Group C** (a product or environment problem first read as a timeout) has 3. **Group D** (unexplained) has 3. Group C is the checks doing their job. The release record already covers the bias of reading those as timeouts. Groups A and B are waste.

The failing test names in the last 200 log lines of all 328 failed jobs since j27000 show no other timing failures. Almost all the rest were work-in-progress branches. For example, 0589's `gencc-first-publisher` failures happened while that branch was changing the gene snapshot warm-up (confirmed).

## Why

1. **Why did the batch gate fail twice?** Two tests armed a short real-time budget (2 s and 3 s) before a child process had started. Each then asserted on something that only exists after startup. At load 14.7 and 19.7 on 18 cores, starting Python and the fake Cargo took longer than that. The production code stopped correctly both times (confirmed, 0667 and the logs).
2. **Why did the tests make that assumption?** A deadline test needs the deadline to run out at a point the test chooses. The quickest way to get that is to pick a small number. Nothing in the repo says otherwise. `AGENTS.md` and `docs/development.md` say nothing about time in tests (confirmed). The repo already has two good patterns, but neither is the default for tests against a real process or connection. Replay mode runs on a virtual clock (`crates/biomcp-fetch/src/deadline.rs:57-79`), and 0667's runner waits until the child is ready and then makes the deadline expire.
3. **Why were they not caught when written?** All three tests arrived with 0457 on 2026-10-30 (confirmed, `git log -S`). Their own gates ran on m5 at load 3.4 to 12 (confirmed). After that they passed every landing gate. At least 128 landings with a `Gate:` trailer ran `docs-check`, which runs these tests (confirmed count; likely each ran them). The failure needs unusual load. Of 616 m5 jobs since j28000, the median start load was 5.7, the 90th percentile 13.6, the 99th 23.1 and the maximum 28.5 on 18 cores (confirmed). The two failing gates sat at about the 90th and 97th percentiles. Gates run each test once, so a test that fails 1 to 2 percent of the time passes for days. Then it breaks whichever batch happens to run at a busy moment. The batch's own changes had nothing to do with it: 0663 and 0665 touched neither `release/mutations.py` nor its test (confirmed).
4. **Why was the class found one test at a time?** Each fix covered the test in front of it. Candidate abbbfd9d8 fixed only the version test, and the next gate failed on its sibling (confirmed). j30393 was never ticketed. 0641 says in writing that it "Defers: other multi-process tests, until one of them fails under load". That rule guarantees each member is found by a failed gate.
5. **Why does nothing stop new ones?** Reviews and proofs run on whatever machine is free. A ticket's proof asks for "run the touched test files whole", once (confirmed, 0667). No check exercises a test under load except two single-purpose cache scripts (below).

## Cause

Tests that use the real clock quietly assume how fast a machine is: that a child starts, a connection opens or planning finishes inside a few seconds, or within 15 percent of one earlier run. The repo has no rule against that assumption and no run that tests it. Gates run each test once, on a shared machine whose load swings from under a third of its cores to 1.6 times its cores. So each such test passes for days, then breaks an unlucky gate, and gets fixed alone.

## What is new compared with the release record

`records/2026-11-05-release-finds-one-at-a-time.md` counts 0622 as release-only "test timing on slow hosts". It sends the cache test to 0641. Both of its fixes (0640 and 0641) target release sections and cache concurrency. What is new here:

- The class also lives in the everyday gate. `release/test_mutations.py` and `release/test_gate.py` run in `make docs-check`. The live-stub fetch tests run in `library-tests`, and the runner's live trials run instrumented in `coverage`. It now breaks batch gates, not only release runs.
- New cases: j30742, j30773, j30393 (never ticketed) and j30338. Plus the older 0103, 0286, 0255, 0272 and 0351, none of which were in the release record.
- `mcp-http/framing-resource-deadlines-and-shutdown` has now needed three timing tickets: 0103, 0286 and 0622. One trial holds about eight real-time sub-checks.
- An inventory of the remaining members on main, below, so they can be fixed as one class.
- 0641's deferral clause is itself a cause.

## At-risk tests on main

**High risk.** The budget has to cover a child process starting, or it is set from one earlier short measurement, and the assertion needs the work finished inside it.

| # | Test | File:line | Why it is at risk | In routine gate? |
|---|---|---|---|---|
| 1 | `test_installed_cli_rows_bind_results_and_fail_on_cli_exit` (`fail=''`) | `release/test_gate.py:161`, through `release/smoke.py:161` (`timeout_seconds = elapsed * 1.15`) and `release/gate.py:149-161` | Channels 2 to 5 run fake suites of 2 to 3 Python starts each, under 1.15 times channel 1's roughly 50 ms. Failed once (j30393, confirmed). The real release gate has the same 15 percent margin over about 258 s, which is likely tight under load (not measured). The 15 percent rule comes from 0429 | Yes (`docs-check`) |
| 2 | `gene-sections/gencc-first-publisher-deadline` | `crates/biomcp/tests/runner/bundle_wait.rs:132` | The 3 s wait starts before the reader's `biomcp` process is spawned. The reader then has a 1 s product budget, so spawn, 1 s and exit must fit in 3 s. The comment at :138-139 guards against false passes, not false failures (likely) | Yes (`coverage`, instrumented) |
| 3 | installed smoke cleanup | `release/test_smoke.py:143` (`timeout=300`) | 300 s against a measured 258 s, about 16 percent headroom. Exceeded once (j30338, cause not proven). 0650 kept "existing 300/60-second limits" on purpose | No (release section) |
| 4 | typed harness preflight | `scripts/typed_harness_preflight.py:54` (3 s for the owner Python to start a child and write its PID), `:82`, `:88`, `:93` (5 s each covering two Python starts and an exchange) | The same shape as 0667. It is run by hand through `scripts/typed_harness_check.sh` (likely) | No |

**Moderate risk.** These run in one process against a local stub with no process start. A budget of 1 s or less starts before connect, TLS or thread start, and the assertion needs that setup done inside it. None has a recorded failure. The risk is judged by reading only.

| # | Test | File:line | Budget and assumption |
|---|---|---|---|
| 5 | `ordinary_backoff_expiry_is_deadline_exceeded_and_releases_permit` | `crates/biomcp-fetch/tests/retries.rs:152` | 100 ms. The test waits for the request at `server.next()` (10 s, `tests/support/mod.rs:149`), so a budget that runs out before the request is sent fails the test |
| 6 | `fresh_hits_skip_quota_and_stale_positives_survive_quota_deadline` | `crates/biomcp-fetch/tests/cache/freshness.rs:30` | 100 ms for a fresh cache hit that must succeed |
| 7 | `local_write_busy_keeps_the_checked_live_reply_and_deadline` | `crates/biomcp-fetch/tests/cache/freshness.rs:112` | 500 ms must cover a live exchange before the write waits out the rest |
| 8 | `source_budget_caps_supplied_deadline_during_headers_and_body` | `crates/biomcp-fetch/tests/transport.rs:368` | 1 s. In the "body" case, headers must arrive in time for status 200 |
| 9 | `deadline_exhausted_during_reply_check_returns_deadline_exceeded` | `crates/biomcp-fetch/tests/transport.rs:421` | 1 s must cover the whole response before the check (asserts 200) |
| 10 | `stream_worker_uses_one_absolute_deadline_in_each_response_phase` | `crates/biomcp-fetch/tests/grpc_worker.rs:340` | 1 s must cover TCP, TLS and HTTP/2 to reach each held phase. Asserts which events were seen |
| 11 | `stream_worker_holds_the_shared_permit_until_stream_cancellation` | `crates/biomcp-fetch/tests/grpc_worker.rs:440`, `:443` | The first stream has 2 s and must deliver a message. The second has 4 s and must wait out the first and then finish |
| 12 | `stream_worker_rejects_callback_completion_after_the_absolute_deadline` | `crates/biomcp-fetch/tests/grpc_worker.rs:515` | 1 s must reach the callback (`assert!(received)`) |
| 13 | `a_stalled_body_keeps_its_partial_bytes_and_resumes_within_a_new_budget` | `crates/biomcp-fetch/tests/bundle/download.rs:175` | 1 s sync budget. Half the body must arrive before it runs out (asserts `bytes == cut`), then a fresh 1 s must finish the resume and commit |
| 14 | `overload` in `mcp-http/framing-resource-deadlines-and-shutdown` | `crates/biomcp/tests/runner/http_limits.rs:225` | 800 ms upper limit on a child server's 503 while it is under load |
| 15 | `cache-resilience/hgnc-first-snapshot-wait-is-bounded` | `crates/biomcp/tests/runner/hgnc_sync.rs:316` | A 30 ms query budget, and the test asserts `Syncing`. A budget already spent when a bundle read starts returns `DeadlineExceeded` (`crates/biomcp-fetch/src/bundle/read.rs:239`), so planning that takes 30 ms fails it. The exact HGNC path was not traced |

**Total at risk: 15 tests (4 high, 11 moderate).** The 0667 fixes leave the three mutation tests safe (`release/test_mutations.py:188-212`, `:312`, `:327`): they now wait for readiness and then expire, or accept both outcomes.

**Low risk, worth watching, not counted.** These are 4 to 5 s guards that cover a child's startup or reply, with a margin of 50 times or more: `runner/http_start.rs:99` (readiness for every HTTP trial), `runner/http.rs:91`, `:164`, `runner/http_limits.rs:12`, `:65`, `:139`, `:159`, `:303`, `runner/held_proxy.rs:290`, `:399`, and `biomcp-test-support/src/grpc.rs:231`, `:438`. Also `scripts/source_capture.py`'s 2 s curl limit against a local stub (`scripts/test_source_capture_plan.py:83`).

### What is fine, and why

- **Replay-mode budgets** run on the virtual clock (`deadline.rs:71-79`). That covers every replayed case YAML and the replay-mode Rust tests (`quotas.rs` replay fixture, `batch.rs`, `record.rs`, `article_search.rs`, `gene_resolution.rs`, adapter replay tests). A busy machine cannot change their result.
- **Short budgets that are meant to run out**, where the assertion does not need earlier progress. Examples: `quotas.rs:106`, `:247`, `grpc_worker.rs:489`, `batch.rs:510`, and offline or input-error refusals in `bundle.rs:95`, `:328`, `:349`, `download.rs:16`, `:37`, `paths.rs:216` to `:353` and `alphagenome_registration.rs:34`.
- **Lower limits**, which a slow machine only makes easier to pass: `quotas.rs:209`, `:326`, `retries.rs:99`, `http_limits.rs:120`, `bundle_sync.rs:113`.
- **Hang guards of 10 s or more**: `WATCHDOG` (30 s), `DEADLOCK_GUARD` (60 s), `server.next()` (10 s), cache-process control (15 s), `bundle_wait` admission (15 to 30 s). Also Python `timeout=20` to `300` around whole scripts in `test_installer.py`, `test_windows.py`, `test_checks.py` and `test_source_revision.py`.
- **In-process local SQLite reads with budgets of 1 to 5 s**: `bundle/native.rs`, `bundle/rejections.rs`, `bundle/rows.rs`, `bundle.rs:82`, `:143`, `:170`.
- **The cache-process test**, after 0569 and 0639, runs on events with 15 s guards. Its 10 s budget starts after the open (`tests/support/cache_process.rs:58`, `:282`).

## Why the routine gate did not catch these before batching

- **Confirmed:** the gate did run them, once per landing, and they passed for six days. These failures are rare and need load. One run per gate cannot be expected to find them.
- **Confirmed:** the failing batch gates ran on m5 at loads of 14.7 and 19.7, beside 5 to 6 other jobs. The author's later focused runs for 0667 ran on gator at 0.12 and 1.8 of 12 cores. The original author's gates for 0457 ran on m5 at 3.4 to 12. So yes, gates usually run on a busier machine than authors' focused runs. But the bigger factor is one run per gate on a machine whose load varies five-fold.
- **Confirmed:** in the gate, the runner's live trials run with coverage instrumentation (`scripts/coverage.sh:104`), which is slower than an author's plain `cargo test`.
- **Confirmed:** j30393 failed on a quiet machine. A budget of "15 percent over one earlier short run" fails on ordinary run-to-run variation, not only under load.

## Existing load or repeat mechanisms

- `scripts/check_cache_process_load.sh` with `scripts/cache_process_load.py` (0569) runs one named test 12 times: 3 waves of 4, with 2 CPU burners inside a 2-CPU container (confirmed).
- `release.smoke.cache_first_opens`, `scripts/check_fixed_cache_first_open.sh` and the other `check_*cache_first_open.sh` scripts (0639) run 50 rounds of 8 installed processes (confirmed).
- 0641 (OPEN): one Make goal to run the multi-process cache checks many times under load, nightly. It covers cache checks only, and it defers everything else until it fails (confirmed).
- pm has `--load-below N`, which avoids load rather than testing under it. Nothing runs an arbitrary test file N times under load (confirmed).

## Fixes, smallest first

1. **Fix the four high-risk tests now, in one test-and-tooling ticket.** Cost: about one small ticket. It prevents the next j30393 and the next failures of rows 2 to 4.
   - **The suite timeout.** Give the derived suite timeout an absolute floor, so it is the larger of 15 percent and a named constant. For example, a 60 s default, configurable. That also protects real release runs from a 15 percent swing. The 15 percent rule is 0429's, so the PM confirms the change.
   - **The gencc reader and the typed-harness preflight.** Start their short waits only after the child reports it has started, and keep the generous guards their sibling waits already use (15 to 30 s).
   - **The installed smoke limit.** 0650 kept the 300 s limit deliberately, so the PM weighs the same floor idea rather than an agent changing it.
2. **Add one narrow rule to the repo's test guidance** (`AGENTS.md` or `docs/development.md`), citing 0667 and this analysis. Suggested wording: "A test that uses real time starts a short deadline only after the setup it is not testing has finished, such as a child starting or a connection opening, or makes the deadline end at a point the test controls. It accepts every valid result of the deadline ending earlier. A real-time limit that the test expects to pass is at least 15 s, or is a lower limit. A limit set from one earlier run has an absolute floor." Cost: minutes. If it had been followed, it would likely have stopped 0457's three tests, j30393 and the bundle_wait gap at review.
3. **Widen 0641 instead of building anything new.** Run the tests that use real time or child processes many times under CPU load of about 1.5 times the cores, matching m5's 99th percentile of 23 on 18. That means the `release/` and `scripts/` Python test files in `docs-check`, the live-stub tests in `biomcp-fetch`, and the runner's live trials by name. Run them nightly, and on demand for a ticket that adds or changes such a test, touched files only, about 10 runs. Reuse `scripts/cache_process_load.py`'s burner-and-waves pattern. Drop the "Defers: other multi-process tests, until one of them fails under load" clause. Any single failure opens a ticket, with no retry, as 0641 already says. Cost: an increment on an open ticket, plus about 30 to 60 minutes of nightly machine time. It would likely have caught 0457's tests within a night of 2026-10-30 instead of in a batch gate on 2026-11-05. The capture rate is an estimate, not measured.
4. **Fix the eleven moderate-risk tests as one later ticket.** Use the same rule: start the budget after setup, use the fixture holds the gRPC and HTTP stubs already have, or accept earlier expiry. Let the widened 0641 run order them. Cost: moderate, Rust test code only. No recorded failures yet, so it can wait.

Considered and not recommended:

- **A lint or ratchet on small time numbers in tests.** Most short budgets in tests are legitimate (expected to run out, or virtual time), so it would be noisy, and it adds a check where the rule plus the load run removes the cause. Revisit if new members keep appearing after fixes 2 and 3.
- **Retries, "flaky" marks or wider budgets as the fix.** Ticket rules forbid them, and they hide real defects like 0639.
- **Running gates with `--load-below`.** It hides the defect, slows the queue, and users and release hosts run loaded.

## Reach

The habit of short real-time budgets in tests is a general one. The shared gate machine runs at up to 1.6 times its cores for every team using pm. That makes it likely other teams pay for the same class. The coordinator should mail Jarvis (`pm send jarvis`; not sent from this read-only analysis). The one-line rule may also belong in the shared outside-in tests skill, which is the rules owner's call.

## Could not verify

- No tests or pm jobs were run (read-only). Failure rates, and whether 10 runs under load would catch each test, are estimates.
- Load figures are the load when the job started, not during the failing test.
- The log scan covered the last 200 lines of the 328 failed jobs since j27000. Earlier cases come from ticket text only, and other unrecorded red-then-green runs may exist.
- The count of 128 gate passes counts landings with a `Gate:` trailer since 0457. It assumes each ran `docs-check`.
- j30338's cause (the 300 s limit) is unproven, as its record says.
- The `hgnc_sync` early-expiry path was not traced. The bundle read entry check suggests it; the snapshot path may differ.
- The moderate-risk list was built by reading code against stated assumptions, not by measurement. The search covered every `Duration`, `Instant`, `sleep`, `recv_timeout`, read-timeout and budget line in `crates/*/tests` and `biomcp-test-support`, and every deadline and timeout in `release/` and `scripts/`. A test that takes its time limit from a helper with a neutral name could still be missed.
- Whether the release gate's 15 percent margin threatens real qualification runs is likely but not measured.
- The causes of 0103, 0255, 0272 and 0351 are still unknown.
- Other repositories were not checked.
