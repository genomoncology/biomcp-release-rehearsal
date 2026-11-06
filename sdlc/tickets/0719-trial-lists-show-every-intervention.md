# 0719 — Trial lists show every intervention

Status: OPEN.

Milestone: 5

## Outcome

A trial row never hides an intervention. Every investigational drug is shown, or the row states how many more there are and how to read them.

## Why

Item 3: lists show only the first 3 interventions, which hid zoldonrasib in NCT07805954. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a case with more than three interventions, in JSON and Markdown, where the last one is the investigational drug.

## Starts from

- Starts from: `crates/biomcp-query/src/trial/` and the trial row renderers.

## Keeps

- Keeps: the size limits on list replies, by a count where a full list cannot fit.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/trial/`, `crates/biomcp/src/`, `tests/`.
