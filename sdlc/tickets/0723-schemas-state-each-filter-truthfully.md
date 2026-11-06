# 0723 — Schemas state each filter truthfully

Status: OPEN.

Milestone: 5

## Outcome

Each MCP and CLI filter takes what its schema and help say. `product_code` works for 510(k) devices, the NCI trial search states or resolves its disease input, the Open Targets section error lists only what can be sent, and source names match across schema, help and server.

## Why

Items 5 and 6: author tool schema allowed `semanticscholar` while help says `semantic-scholar`; `product_code` rejected for 510(k); NCI search needs an NCIt code though the schema says "Disease or NCIt"; Open Targets takes one section but the error lists both. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a case per named filter, plus a generated check that every schema enum and every filter a command advertises is accepted by that command, so this class fails in the gate. `crates/biomcp-adapters/src/fda_510k/` already reads `product_code`.

## Starts from

- Starts from: `crates/biomcp-types/src/`, `crates/biomcp-adapters/src/fda_510k/`, typed catalog `crates/biomcp/src/mcp/typed/catalog.rs`.

## Keeps

- Keeps: 0683 tool budget.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-types/`, `crates/biomcp-adapters/`, `crates/biomcp-query/`, `crates/biomcp/`, `tests/`.
