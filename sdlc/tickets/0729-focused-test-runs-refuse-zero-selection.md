# 0729 — Focused test runs refuse zero selection

Status: OPEN.

Milestone: 5

## Outcome

A focused test run that selects no tests fails loudly and says which option to use. Focused builds use the existing Cargo JSON artifact selection, with an explicit runner, executable and case root, and the help says which missing argument triggers a rebuild.

## Why

Focused-artifact RCA (m28816): three tickets paid for five dependency-heavy builds, 26m55 of gross compiler time, avoidable part unmeasured. In j32380, `--proofs` used for the `--cases` catalog selected zero tests, and a developer caught it by hand. This is separate from 0712, which is about folder paths inside release binaries.

## Proof

- Proof: a run with the wrong selector fails with the right option named, failing first; a focused run reuses its build.

## Starts from

- Starts from: the focused runner in `tools/` and the coordinator's RCA report.

## Keeps

- Keeps: existing `--cases` and `--proofs` meanings.

## Defers

- Defers: nothing.

## Changes

- Changes: `tools/`, `docs/development.md`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

The ticket's `tools/` is not a folder. The change sits in `crates/biomcp/tests/tools.rs`, `crates/biomcp/tests/runner/case_main.rs`, `crates/biomcp/tests/runner/proof.rs`, `scripts/trials.py` and `scripts/test_trials.py`, plus `docs/development.md`.
