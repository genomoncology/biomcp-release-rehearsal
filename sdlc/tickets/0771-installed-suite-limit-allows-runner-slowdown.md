# 0771 — Installed suite limit allows runner slowdown

Status: COMPLETE.
Landed: 83c9ce837cfe3b85900263136e40c4ae81702c3b

Milestone: 5

## Outcome

The installed-gate case suite does not fail a release because a shared runner slowed down between install channels, and it still stops a suite that hangs.

## Why

Practice release run 42464420823 at c33d2c330 failed in the Intel macOS `qualify` job: `case-runner --test-threads 2` "timed out after 460.9 seconds" on the fifth channel, `download-install-script`. The first channel's suite took about 400 s, and `release/gate.py:156-158` sets every later channel's limit to `max(60, first elapsed * 1.15)` (`release/smoke.py:176`). The second and third channels took 403 and 404 s; the fourth took about 620 s, so the runner itself slowed. Every case passed on each channel that finished. A limit 15% above one sample of a shared runner is a timing test, and a timing test is fixed, not rerun.

## Proof

- Proof: a unit check shows a later channel that takes 1.6 times the first channel's time passes, and a channel that never finishes is still stopped with a clear message. It fails on main today.

## Starts from

- Starts from: `release/gate.py` `installed` and `check` (lines 140-162) and `release/smoke.py` `case_suite` and `timeout_seconds` (lines 155-180).

## Keeps

- Keeps: the absolute floor, the per-case checks, and every channel the gate installs.

## Defers

- Defers: nothing.

## Changes

- Changes: the limit rule and its unit check. Prefer a simple rule, such as a fixed multiple of the first sample with a generous ceiling, over new measurement.

## Depends on

- Depends on: nothing.
