# 0718 — Sources show how old their data is

Status: OPEN.

Milestone: 5

## Outcome

Every reply from a source with a known release or data date states that date, and old data is flagged in the reply. Where BioMCP holds a newer fact from a source of record, such as an FDA approval against an Open Targets clinical phase, the reply shows both. Recovery advice never offers a step the current server refuses, such as `sync` on a hosted server.

## Why

Items 2 and 10: Open Targets listed sotorasib in colorectal cancer as max phase 3 after the FDA approval of 2025-01-16, with no flag; EMA data is marked stale and the suggested fix is `sync`, which hosting refuses. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a recorded-reply case per dated source family; a case with an Open Targets phase and a later FDA approval; a hosted case where stale data offers no `sync`. Age limits are configurable with a default. No claim of which source is right beyond showing both with dates.

## Starts from

- Starts from: source registry in `crates/biomcp-adapters/src/registry/`, `crates/biomcp-query/src/next.rs`, `crates/biomcp/src/mcp/hosted.rs`.

## Keeps

- Keeps: source values as published. Hosting's snapshot stays readable: no store format change without PM sign-off.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/`, `crates/biomcp-query/`, `crates/biomcp-types/`, `docs/`, `tests/`.

## Amendment, 2026-11-06

PM ruling (m29065): the comparison shows Open Targets phase facts beside dated FDA approval events, and says their indications are not linked. Nothing is labeled as superseding a phase on the drug name alone. Unknown dates stay unknown. A same-indication lookup is not part of this ticket; if the evaluation shows the need, it gets its own ticket.
