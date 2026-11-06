# 0725 — Paging never offers an empty page

Status: OPEN.

Milestone: 5

## Outcome

A reply offers a next page only when one exists. When all rows of a known total are returned, no next page appears.

## Why

Item 11: sponsor search offered a next page after 20 of 20; the next page was empty. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a case at exactly the total, one row short, and an unknown total, for each paging family. Works with the end-of-results fix in 0693.

## Starts from

- Starts from: `crates/biomcp-query/src/next.rs` near line 94 and `window.has_more()`.

## Keeps

- Keeps: stored pool behavior.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/`, `tests/`.
