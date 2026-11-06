# 0694 — Tool proofs run in the routine gate

Status: OPEN.

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
