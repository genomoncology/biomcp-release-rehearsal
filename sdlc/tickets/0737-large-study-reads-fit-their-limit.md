# 0737 — Large study reads fit their limit

Status: OPEN.

Milestone: 5

## Outcome

`get study ID sample-lists` and the study rows read answer for the largest cBioPortal studies, or say clearly that the reply was too large and what to read instead.

## Why

Review of 0721: real study-wide sample list replies are 2.5 MB for `msk_chord_2024`, 2.58 MB for `msk_met_2021` and 5.43 MB for `msk_impact_50k_2026`, above the 1,526,432-byte limit. 0721 fixed the recurrence path by reading the selected list alone. The study reads keep that limit, so they should fail on these studies. This is read from the code and not yet run.

## Starts from

- Starts from: `crates/biomcp-adapters/src/cbioportal/`, `crates/biomcp-query/src/study/`, with one live call first.

## Keeps

- Keeps: member IDs never reach a reply, fixture or the repo (0238). No store format, cache namespace or BioData pin change.

## Changes

- Changes: `crates/biomcp-adapters/src/cbioportal/`, `crates/biomcp-query/src/study/`, `tests/cases/study/`.

## Proof

- Proof: `get study msk_impact_50k_2026 sample-lists` through the built CLI, live.

## Defers

- Defers: nothing.

## Depends on

- Depends on: 0721.
