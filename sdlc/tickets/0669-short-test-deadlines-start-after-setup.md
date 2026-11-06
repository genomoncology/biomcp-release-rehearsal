# 0669 — Short test deadlines start after setup

Status: COMPLETE.
Landed: b303929aa5547cc0943b1652809d6d4f2ff69df7

Milestone: 5

## Outcome

The four high-risk tests in `records/2026-11-05-timing-tests-under-load.md` no longer assume how fast the machine is, and the repo's test guidance says how to write a real-time test. A busy gate machine cannot fail them while the product behaves correctly.

1. The release gate's derived suite timeout (`release/smoke.py:161`, `release/gate.py:149-161`) is the larger of 1.15 times the first channel's time and an absolute floor. The floor is a named, configurable constant with a 60-second default. PM ruling: this keeps 0429's 15% rule and adds the floor.
2. `gene-sections/gencc-first-publisher-deadline` (`crates/biomcp/tests/runner/bundle_wait.rs:132`) starts its short wait only after the reader process reports it has started.
3. The typed harness preflight (`scripts/typed_harness_preflight.py:54`, `:82`, `:88`, `:93`) starts its short waits only after each child reports it has started.
4. The installed smoke limit (`release/test_smoke.py:143`, 300 s against a measured 258 s) becomes a named, configurable hang guard with a 600-second default. PM ruling: this limit exists to catch hangs, not to measure speed, so it needs real headroom. This replaces 0650's choice to keep 300 s.
5. `docs/development.md` gains one paragraph: a test that uses real time starts a short deadline only after the setup it is not testing has finished, such as a child starting or a connection opening, or makes the deadline end at a point the test controls. It accepts every valid result of the deadline ending earlier. A real-time limit the test expects to pass is at least 15 s or is a lower limit. A limit set from one earlier run has an absolute floor. Cite 0667 and the record.

## Why

Tests that start a short timer before a child or connection is ready broke two batch gates in one evening (0667), and j30393 failed the same way on a quiet machine without a ticket. These four are the remaining high-risk members on main (`records/2026-11-05-timing-tests-under-load.md`). Two of them run in every routine gate, so they belong before the 1.0 cut.

## Proof

- Proof: each test keeps the assertion it exists for: the gate still fails on a CLI exit and on a real overrun past the floor; the gencc reader still proves its deadline; the preflight still fails a dead child; the smoke still fails a hang. Run each touched test file 10 times while CPU burners hold load at about 1.5 times the cores, reusing `scripts/cache_process_load.py`'s burner pattern, with no failure. Then the routine gate.

## Starts from

- Starts from: the file and line references above, the record's high-risk table, and 0667's readiness pattern in `release/test_mutations.py`.

## Keeps

- Keeps: production deadlines and 0429's 15% rule above the floor. No retries, no flaky marks, no fake clock over real I/O, no skipped deadline coverage.

## Defers

- Defers: the eleven moderate-risk tests to 0670; repeated load runs of every real-time test to 0641.

## Changes

- Changes: `release/smoke.py`, `release/gate.py`, `release/test_gate.py`, `release/test_smoke.py`, `crates/biomcp/tests/runner/bundle_wait.rs`, `scripts/typed_harness_preflight.py`, `docs/development.md`.
