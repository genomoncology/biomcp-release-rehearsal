# 0755 — MyChem resolves acetaminophen

Status: READY.

Milestone: 5

## Outcome

`get drug acetaminophen --source mychem` answers, and `acetaminophen` and `paracetamol` give the same trial total.

## Why

0741's lane, 2026-11-07: `acetaminophen` gives 4700 trials and `paracetamol` 4787, because each word resolves through a different source. Probe j34592 showed `get drug acetaminophen --source mychem` failing with a bad reply, while a direct MyChem query finds one drug structure for the word. So the MyChem lookup fails and only ChEMBL's names are used.

## Starts from

- Starts from: the MyChem drug adapter and `crates/biomcp-query/src/drug/search_names.rs`. Find which step gives the bad reply before changing anything.

## Keeps

- Keeps: 0717's lookup order and 0741's official names. Tune nothing to these two words: fix the general cause.

## Changes

- Changes: the cause of the bad reply.

## Proof

- Proof: a recorded-reply case with the real MyChem reply shape that failed, and one live check of the two totals.

## Defers

- Defers: drugs whose two sources disagree on names for other reasons.

## Depends on

- Depends on: 0741.

## Evidence

- Evidence: j34592, j34540.

## Progress
