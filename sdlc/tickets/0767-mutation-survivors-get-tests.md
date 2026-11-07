# 0767 — Mutation survivors get tests

Status: OPEN.

Milestone: 5

## Outcome

The three mutants that survived the release mutation section are caught by outside-in tests, and the mutation section passes when rerun alone.

## Why

0640's preflight at 4983fb9dc: the mutation section (j35185, j35194, j35244) picked 22 mutants. 8 were caught, 3 survived and 11 did not compile. The calibration sample hit the same 3. The cause is gaps in the product's tests, not the tooling.

## Proof

- Proof: each new case fails with its mutant applied and passes without it. The mutation section then passes on its own at the landed commit.

## Starts from

- Starts from: the survivors in the mutation report kept with the 0640 run data:
  1. `allele_registry/mod.rs:75`: an error reply, or a reply for the wrong allele, must not be cached.
  2. `adverse_event_sources/components.rs:21`: a MAUDE search is refused when the GMDN terminology is turned off.
  3. `adverse_event_sources/components.rs:62`: the second terminology of a source that carries two is labelled.

## Keeps

- Keeps: product behaviour. This adds tests only, unless a test shows a real bug.

## Defers

- Defers: nothing.

## Changes

- Changes: cases or unit tests near those files.

## Depends on

- Depends on: nothing.
