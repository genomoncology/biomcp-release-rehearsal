# 0722 — Author search works from open sources

Status: OPEN.

Milestone: 5

## Outcome

At `restricted`, author search finds researchers and their papers from sources BioMCP already holds and may serve, such as PubMed and Europe PMC, and says what it could not establish. Semantic Scholar and ORCID still add to it where they are enabled.

## Why

Finding key opinion leaders row: `search_author` fails on hosting because Semantic Scholar is noncommercial and off. Pharma needs this. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a recorded-reply case at `restricted` with Semantic Scholar and ORCID off: a name, a topic, and two same-named authors reported as unresolved, not merged. No new source; a new one goes through `planning/sources-to-add.md`.

## Starts from

- Starts from: `crates/biomcp-query/src/author/`, `crates/biomcp-types/src/author/query.rs`.

## Keeps

- Keeps: noncommercial sources stay off at `restricted`.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/author/`, `crates/biomcp-types/`, `tests/`.
