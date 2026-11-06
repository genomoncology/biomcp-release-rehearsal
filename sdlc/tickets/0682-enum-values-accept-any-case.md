# 0682 — Enumerated values accept any letter case

Status: COMPLETE.
Landed: 7a0cab3c7f89fa673ce858821bd3962523bedf57

Milestone: 5

## Outcome

An enumerated value such as trial `status` accepts any letter case (`recruiting`, `RECRUITING`) and echoes the canonical form; a value that matches no choice in any case is still refused with the valid list.

## Why

Hosting, alongside Claude's retest of the live typed server on 2026-11-06 (pin 1bb97e64; hosting m27741 via Jarvis m27744; evidence in `data/biomcp-hosting-evidence/2026-11-05-claude-eval-v2/`): `search_trial` rejected lowercase status `recruiting`. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Investigate first: reproduce on current main in typed mode (the retest ran an older pin). If main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Then a structural test that every enumerated parameter accepts a lowercased and an uppercased choice in CLI and typed mode, and a refusal case. Typed schemas may keep the canonical enum and document case-insensitivity. Routine gate.

## Starts from

- Starts from: the shared choice parser and typed schema enums.

## Keeps

- Keeps: canonical values in output, schema IDs and the refusal for unknown values.

## Defers

- Defers: nothing.

## Changes

- Changes: the shared argument parser, typed schema generation, `tests/`.
