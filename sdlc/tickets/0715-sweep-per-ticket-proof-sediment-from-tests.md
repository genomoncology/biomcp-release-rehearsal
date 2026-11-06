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
