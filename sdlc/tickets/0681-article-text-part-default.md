# 0681 — Article text defaults to the abstract and lists valid parts

Status: OPEN.

Milestone: 5

## Outcome

`article_text` (and `get article`) defaults `part` to the abstract when it is omitted, and a bad `part` error lists the valid values as values, not a sentence.

## Why

Claude's retest of the live typed server on 2026-11-06 (pin 1bb97e64; hosting m27741 via Jarvis m27744; evidence in `data/biomcp-hosting-evidence/2026-11-05-claude-eval-v2/`): `part` had no default, and its error listed a sentence as the valid value. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Investigate first: reproduce on current main in typed mode (the retest ran an older pin). If main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Then cases for omitted `part`, each valid part, and a bad part whose error lists exactly the valid values. Routine gate.

## Starts from

- Starts from: the article command's part argument and the shared usage-error builder from 0652.

## Keeps

- Keeps: full text stays opt-in.

## Defers

- Defers: nothing.

## Changes

- Changes: the article query and its schema, `tests/`.

## Amendment, 2026-11-06

PM ruling (m27763): current main's `part` is a one-based number that selects from the article's body outline, alongside the find, variant and save modes; there is no abstract or full-text enum. Keep that syntax. An omitted selector returns the existing `get article` card with its abstract. An invalid `part` states the valid range and how to list the outline, and puts no sentence in `valid_values`. No new enum values.
