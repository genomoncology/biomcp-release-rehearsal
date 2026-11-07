# 0762 — Release Linux job installs pinned Python

Status: COMPLETE.
Landed: a9e03838c9699fff5730b2ee6fd9cfbd991040fa

Milestone: 5

## Outcome

Every release job that runs `release/toolchain.py python-check` first installs the pinned Python, so a change in the hosted runner's default Python cannot fail qualification.

## Why

Practice release run 42401111058 at 4983fb9dc failed in 3 seconds in the `qualify` job for `manylinux_2_28_x86_64` on `ubuntu-24.04`: "Python differs from release tool pin: 3.13". The workflow skips `actions/setup-python` on every Linux job, yet runs the Rust toolchain step, which checks Python, on that one Linux job. The pin and workflow lines are unchanged since 2026-10-28 and the same job passed on 2026-11-05, so the runner image's default Python changed. The release relied on it.

## Proof

- Proof: a practice release on the public rehearsal repo at the landed commit passes the `qualify` job for `manylinux_2_28_x86_64`, per `decisions/2026-11-05-release-finds-no-bugs.md`. A local check that the workflow's setup-python condition covers every job that runs `python-check` fails on main today.

## Starts from

- Starts from: `.github/workflows/release.yml` lines 109-117, where setup-python has `if: runner.os != 'Linux'` and the toolchain step has `if: runner.os != 'Linux' || matrix.platform == 'manylinux_2_28_x86_64'`. Check every other job that calls `python-check` or uses `python` from the host.

## Keeps

- Keeps: Linux container jobs that build inside the manylinux image; every pin; action pins by commit.

## Defers

- Defers: nothing.

## Changes

- Changes: `.github/workflows/release.yml` and the existing workflow check, if one reads these conditions.

## Depends on

- Depends on: nothing.
