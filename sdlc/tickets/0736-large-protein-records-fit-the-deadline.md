# 0736 — Large protein records fit the deadline

Status: OPEN.

Milestone: 5

## Outcome

`biomcp get protein` returns for large, heavily annotated entries such as EGFR (P00533) within the source deadline, on a normal day. When UniProt is slow, the reply says so and offers the sections it can still give.

## Why

The live check (0732) failed `get protein P00533` on both tries in one run of four, and `search protein` in another, with `uniprot: deadline-exceeded`. Fetched directly, the full entry is 385 KB and took 27 seconds, past the product's 20-second limit, while a small request took about a second. Whether the product fetches that full entry is not yet confirmed. EGFR is a core cancer gene, so agents will ask for it.

## Starts from

- Starts from: the UniProt adapter and `crates/biomcp-query/src/protein/`, with a live timing of what the product asks for.

## Keeps

- Keeps: the sections and fields callers get, UniProt's rate limits, and recorded cases. No store format, cache namespace or BioData pin change.

## Changes

- Changes: `crates/biomcp-adapters/`, `crates/biomcp-query/src/protein/`, `tests/cases/protein/`.

## Proof

- Proof: `get protein P00533` and `search protein` through the built CLI, live, several times, with their times. No swap to a smaller protein in the live check.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.
