# 0715 — Sweep per-ticket proof sediment from tests

Status: OPEN.

Milestone: 5

## Outcome

Tests that exist only to prove a closed ticket, and repeat a stronger outside-in case, are gone. Coverage of behavior stays the same.

## Why

Row W39 of `records/2026-11-06-pre-release-review.md`, first deferred past release. Ian, 2026-11-06, wants the project clean.

## Proof

- Proof: for each test removed, the case that still covers its behavior, and the coverage floor in `coverage.floor` holds. The mutation section passes on the result.

## Starts from

- Starts from: `tests/`, `crates/biomcp/tests/`, `skills/outside-in-tests.md`.

## Keeps

- Keeps: every test that alone proves a behavior, and every regression test for a real defect.

## Defers

- Defers: nothing.

## Changes

- Changes: `tests/` and `crates/*/tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

This ticket's evidence is Make goals and scripts. The source finding counted goals named for single tickets and scripts nothing calls, and it sampled tests only lightly. The scope becomes, in order:

1. Scripts and Make goals with no caller. A name search gives a starting list, not a delete list: a script named by an open ticket stays until that ticket closes.
2. Tests, one folder at a time and time-boxed. Stop at the first folder that gives nothing.
3. Machine names and workspace paths in tracked files outside `sdlc/`, which 0279 cleared and which have come back. Add one search for them to `scripts/check_docs.py`.

This runs last among cleanup, after the bench and hosting commits, because every lane adds tests.

The Proof changes, because the current one cannot fail. The coverage floor sits about 4.7 points below measured coverage. The mutation run counts survivors and timeouts as passing, and a test-only change leaves only the rule files in its scope. New Proof: coverage after the sweep is no lower than on the commit before it, less 0.2 points; each removed test is listed with the case that covers the same behavior; and the routine gate passes. The mutation clause is dropped.

Step 1 and step 3 start now, after the landing check ticket lands: Ian has approved deleting single-ticket Make goals and scripts. Step 2 stays last.
