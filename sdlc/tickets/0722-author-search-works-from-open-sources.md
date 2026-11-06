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

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

The product work is done and passed the routine gate. A script that re-checks its own build artifact does not land: no review tooling, host paths or pinned hashes in the repo. Land from the gated commit with main merged in.

Europe PMC stays out of this landing. If the evaluation shows that a list of papers with author names does not answer the key opinion leader workflow, affiliations and ORCIDs from Europe PMC get their own ticket.
