# 0691 — Article pools publish only whole answers

Status: OPEN.

Milestone: 5

## Outcome

A partial article search, edge or recommend result is kept for its own pages but never reused as the answer to a new search. A stored pool whose sources are now disabled does not fail a new search; the disabled set is part of the pool key. Explicit pool IDs keep their failing branch.

## Why

Rows W2 and W19 of `records/2026-11-06-pre-release-review.md`. `article/mod.rs:75-78,130` stores a partial answer for 24 hours, with copies in `edges.rs` and `recommend.rs`, and hosted strips `--fresh` (`hosted.rs:138`). One source's bad minute shapes a day of answers. Both reproduced in j32005.

## Proof

- Proof: cases, failing first, for a partial search followed by a new search that reaches the source again in all three places, and a stored pool plus a newly disabled source. Routine gate.

## Starts from

- Starts from: the lines above and `decisions/2026-10-23-the-cache-is-a-local-sqlite-database.md`.

## Keeps

- Keeps: stable paging within a pool. No migration (`decisions/2026-11-01-no-backwards-compatibility.md`).

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/article/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

This ticket merges after the licensed article text and search drug names tickets. Compute whether to publish after drug name reuse has run, so a rebuilt pool becomes current. Keep one term resolver, the one from search drug names.

A pool is published as whole only when no license or drug name lookup failed.

The mapping check script and per-ticket Makefile goals do not land.
