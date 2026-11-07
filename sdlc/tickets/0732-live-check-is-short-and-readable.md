# 0732 — Live check is short and readable

Status: COMPLETE.
Landed: af4740e1acd702cf648c5fdddf165c7ad36fd2a5

Milestone: 5

## Outcome

One short scheduled check runs a few real CLI commands against live sources, one or two per source family, in minutes. Each failure prints the command and the reason in one line, and any failure mails the PM. The large classifier and its drift reports are deleted.

## Why

The nightly live check failed 11 nights running with no readable reasons, and nobody acted. Passing recorded cases also missed what Ian's live agent test found. A scheduled check runs only if its failure is readable and has an owner (`skills/build-efficiency.md`). The PM owns this one.

## Proof

- Proof: one run on main with its output, and one run with a source pointed at a bad address, showing the one-line reason and the mail. KEGG and the LitSense licensed subset stay uncalled.

## Starts from

- Starts from: `scripts/nightly_live_check.py`, `scripts/nightly_test_support.py`, `scripts/nightly_recommendation_cases.py`, and whatever schedules them.

## Keeps

- Keeps: recorded cases, and rate limits.

## Defers

- Defers: nothing.

## Changes

- Changes: `scripts/`, `Makefile`, `docs/development.md`.
