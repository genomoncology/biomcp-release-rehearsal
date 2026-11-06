# 0726 — Cell line search ranks and filters for research

Status: OPEN.

Milestone: 5

## Outcome

Parent cell lines rank above derived lines, a caller can exclude derived lines, and a caller can search by gene or mutation where Cellosaurus records one.

## Why

Item 12: derived lines such as "AsPC-1 SPINK1 #10" crowd the results, and KRAS G12D lines cannot be searched. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then cases for ranking, the derived filter and a mutation search from Cellosaurus fields. If the bundled data lacks sequence variations, report it to the PM for `planning/sources-to-add.md`. No engineering of lines; this is lookup only.

## Starts from

- Starts from: `crates/biomcp-adapters/src/cellosaurus/`, `crates/biomcp-query/src/cell_line/`.

## Keeps

- Keeps: Cellosaurus terms.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/src/cellosaurus/`, `crates/biomcp-query/src/cell_line/`, `crates/biomcp-types/`, `tests/`.
