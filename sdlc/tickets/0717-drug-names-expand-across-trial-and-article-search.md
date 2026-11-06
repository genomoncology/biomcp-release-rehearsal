# 0717 — Drug names expand across trial and article search

Status: OPEN.

Milestone: 5

## Outcome

A trial or article search on a drug name also searches that drug's other names, so `sotorasib`, `AMG 510` and `Lumakras` give the same set. The reply lists the names it searched, and a caller can ask for the exact term only.

## Why

Item 1: `sotorasib` found 73 trials, `AMG 510` 40, `Lumakras` 16, all three together 75; `daraxonrasib` 91 papers and `RMC-6236` 102. The report calls this silent incompleteness, worse than an error for pharma. 0661 joined names across drug sources but not trial and article search. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then recorded-reply cases for a brand, generic and code name returning one set, with the searched names in the reply, and a name that resolves to nothing searching as typed. Names come from drug sources BioMCP already reads, as in 0661. Ambiguous names do not expand; the reply says so.

## Starts from

- Starts from: `crates/biomcp-query/src/drug/`, `crates/biomcp-query/src/trial/`, `crates/biomcp-query/src/article/`, 0661.

## Keeps

- Keeps: exact-term search, and paging over a stored pool.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/`, `crates/biomcp-types/`, `docs/`, `tests/`.
