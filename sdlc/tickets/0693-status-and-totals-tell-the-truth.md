# 0693 — Status and totals tell the truth

Status: OPEN.

Milestone: 5

## Outcome

A good trial page stays partial, not failed, when a side call fails. Paging past the end of an openFDA search neither loosens to a word match nor reports a false zero, and FAERS past 25,000 gets a clear refusal. Protein cards state counts and a next step when they cut rows. OpenTargets drug paging counts source rows. Label search by application number keeps total and rows in step. A moved cache can resume an interrupted sync.

## Why

Rows W12 to W16, W25 and W27 of `records/2026-11-06-pre-release-review.md`. Measured live on openFDA: j32050 and j32056 (METFORMIN HYDROCHLORIDE exact 190, word 212; five other searches return 404 that reads as total 0). `trial.rs:93` overwrites partial; `protein/sections.rs:33-52`; `drug/native.rs:126`; `bundle/store.rs:50-66`.

## Proof

- Proof: one failing test or replayed case per item, then the fix. Drop any W27 adapter item that has no failing test. Sequence the protein item after 0680. Routine gate.

## Starts from

- Starts from: the lines above; one shared past-the-end helper for the seven openFDA adapters.

## Keeps

- Keeps: existing correct results.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/`, `crates/biomcp-adapters/src/`, `crates/biomcp-fetch/src/bundle/`, `tests/`. Split by item if lanes collide.
