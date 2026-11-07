# 0694 — Tool proofs run in the routine gate

Status: COMPLETE.
Landed: 86658b922

Milestone: 5

## Outcome

The routine landing gate runs the tool proofs, so a proof that only the installed release ran can no longer stay red on main. The gate's Python version is pinned in one place and every script uses it. Gate failures print why, including `check_front_map.py:86`.

## Why

Rows W6, W36 and W40 of `records/2026-11-06-pre-release-review.md`. Markdown help reached the hosted rehearsal because `landing-gate` never runs `--proofs` (`Cargo.toml:49-51`). Python is 3.12 in `release/toolchain.json`, `python3.13` in 36 files and 3.9.6 by default on m5, which cost j31827, j30853 and j32002. Ruling: `decisions/2026-11-05-release-finds-no-bugs.md`.

## Proof

- Proof: a deliberately broken proof fails the gate and the log names it. Record the added gate time (at most about 4.5 minutes, from j32033 and j32021). After 0686 lands, rerun the installed gate once to settle whether a second cause exists.

## Starts from

- Starts from: `Makefile`, `release/toolchain.json`, `scripts/check_front_map.py`.

## Keeps

- Keeps: gate strength. Mac-only routine gate stays (`decisions/2026-11-05-release-finds-no-bugs.md`).

## Defers

- Defers: nothing.

## Changes

- Changes: `Makefile`, `release/`, `scripts/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

First step: one job running `cargo test --locked -p biomcp --test tools -- --proofs` on main, with the environment `release/smoke.py` sets. The last run of this form was before 0686 landed: 3 of 38 failed (`mcp-http/readiness-rejects-eof-and-hung-child`, `live-source-checks/retained-drift-witnesses`, `schema-discovery/rejects-bad-cases`). Each remaining failure is fixed or ticketed before the stage joins the gate.

`release/toolchain.json` says Python 3.12, while `sdlc/pm.json` and 36 files say 3.13. Set the toolchain to 3.13, so the routine gate and open lanes stay unchanged and only the hosted side moves. Do the file sweep after the bench build.

The bare asserts in `scripts/check_front_map.py` are now at lines 73, 74, 78 and 87.

Ian's 2026-11-06 ruling replaces the Outcome's routine gate: tool proofs run in the final release suite, not at landing. This ticket fixes or deletes the red proofs, and nothing more.
- 2026-11-07 Closed by PM. All tool proofs pass in the smoke environment (j34179), and the installed gate passed on all five platforms in practice run 42364969543 at 160521f5d, including `staged-payload` and staged Homebrew. The Python pin and `check_front_map.py` items moved to 0745.
