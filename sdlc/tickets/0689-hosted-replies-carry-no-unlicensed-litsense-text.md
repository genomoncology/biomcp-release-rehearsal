# 0689 — Hosted replies carry no unlicensed LitSense text

Status: OPEN.

Milestone: 5

## Outcome

At hosted `restricted`, an article search with `source=litsense` keeps its PMID ranking and returns no `sentence` text. LitSense is labeled `per-record`. Any route that returns article text at `restricted` applies the per-record rule, including stored pools.

## Why

Row W1 of `records/2026-11-06-pre-release-review.md`: `litsense/mod.rs:60-70` returns body sentences with no license check, and recorded replies include noncommercial papers. That breaks Ian's ruling in `decisions/2026-11-04-hosted-source-modes.md`: "data elements ... we will not access them, share them, or return them back." Read in code, not run, because LitSense stays uncalled.

## Proof

- Proof: a recorded-reply hosted case at `restricted`, failing first, whose LitSense reply holds a sentence and asserts no sentence text reaches the caller, plus one for a stored pool (`article/page.rs:66-90`). Check whether PubTator returns passages the same way. No live LitSense call. Routine gate.

## Starts from

- Starts from: `crates/biomcp-adapters/src/litsense/mod.rs`, `disabled_use(Use::PerRecord)`, `pmc_oa/rights.rs`.

## Keeps

- Keeps: local and CLI use unchanged, and the ranking.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/src/litsense/`, `crates/biomcp-query/src/article/`, `tests/`.
