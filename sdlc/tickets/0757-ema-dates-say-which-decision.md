# 0757 — EMA dates say which decision they are

Status: READY.

Milestone: 5

## Outcome

Each date on an EMA medicine record says which event it marks, in words taken from EMA's own description of the field, so a reader can tell a first authorisation from a later decision. A date is exactly the day EMA's data gives.

## Why

Evaluation, 2026-11-07: on the EMA tecovirimat question, all three BioMCP modes gave the wrong European Commission decision date. The record carries one decision date for the product, and the model took it as the answer to which decision was asked about. Whether the date itself is off by a day is not yet known.

## Starts from

- Starts from: `crates/biomcp-adapters/src/ema/map.rs`, which passes every native key ending in `_date` through as an authorisation date with its key as the only label, and `ema/raw.rs` `date()`.

## Keeps

- Keeps: native key names and values; no date is guessed or derived.

## Changes

- Changes: each EMA date carries a short label from EMA's published field description (for example first authorisation, or latest decision), and the card's text says so. Check the date parser against one public EMA record: the date shown equals the day in EMA's data, with no time-zone shift.

## Proof

- Proof: a case on a public EMA reply shape shows each date with its label in JSON and Markdown, and one date that sits near midnight in the source keeps its day.

## Defers

- Defers: listing every past decision, unless EMA's data already carries them.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: Sonnet evaluation of the 18 questions, 2026-11-07.

## Progress
