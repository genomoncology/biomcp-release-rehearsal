# 0751 — A slow name lookup leaves article search whole

Status: COMPLETE.
Landed: 91870c431f466a00a75db1908ccdbd32a6228c54

Milestone: 5

## Outcome

When the drug name lookup times out, article search still returns its articles and says the lookup was skipped.

## Why

Nightly j34395: `search article 'AKT1 E17K'` passed only on its second try, and the first try reported `openfda-label: deadline-exceeded`. 0717 added that lookup. An article search on a gene variant should not depend on a drug label source.

## Starts from

- Starts from: `crates/biomcp-query/src/drug/search_names.rs`, `crates/biomcp-query/src/article/drug_names.rs`.

## Keeps

- Keeps: 0717's name expansion when the lookup answers in time; 0749's key rules.

## Changes

- Changes: first read what the first try returned. If the lookup can fail the command or use up its time, give it a bounded share of the budget and carry on with the typed term.

## Proof

- Proof: a recorded-reply case: a stalled lookup still gives articles and a notice, inside the command's time budget.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: j34395 log, 2026-11-07.

## Progress
