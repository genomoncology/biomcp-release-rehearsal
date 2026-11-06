# 0721 — Variant recurrence accepts what it says it accepts

Status: OPEN.

Milestone: 5

## Outcome

Every variant form the error message and schema list works in `variant_recurrence`, and a cohort mutation frequency comes back for a common variant. Forms it cannot use are not listed.

## Why

Mutation frequency row of the test: the tool rejected every variant form tried, including forms its own error lists. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a case per listed form through CLI, MCP and the typed tool, and a test that the listed forms and the accepted forms come from one owner. Must work on hosting's existing cBioPortal snapshot.

## Starts from

- Starts from: `crates/biomcp-query/src/variant/recurrence.rs`, `crates/biomcp-query/src/variant/recurrence/matching.rs`, `crates/biomcp-types/src/query/recurrence.rs`.

## Keeps

- Keeps: 0710 study flags.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/variant/`, `crates/biomcp-types/`, `tests/`.
