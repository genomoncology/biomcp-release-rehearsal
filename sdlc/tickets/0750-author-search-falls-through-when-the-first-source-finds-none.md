# 0750 — Author search finds papers when the first source finds none

Status: COMPLETE.
Landed: 380f47d396a8c3ffedcb357574fc9ae922a0b698

Milestone: 5

## Outcome

Under default settings, `search author` with a topic gives the open sources a turn when Semantic Scholar finds nothing.

## Why

The bench hand-off check at c821e374d: `search author "KRAS G12C"` returned 0 with status complete from Semantic Scholar alone, while `--source pubmed` found 2093. 0722 fixed this only where Semantic Scholar is off. A user with default settings gets an empty answer that reads as complete.

## Starts from

- Starts from: `crates/biomcp-query/src/author/`.

## Keeps

- Keeps: 0722's restricted behavior; name searches; no merging of same-named authors.

## Changes

- Changes: the automatic source order tries PubMed when the first source returns no records, and says which source answered.

## Proof

- Proof: a recorded-reply case: an empty Semantic Scholar page followed by PubMed records gives those records, and a name Semantic Scholar finds makes no PubMed call.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: j34416 and j34425 on the M5 at c821e374d.

## Progress
