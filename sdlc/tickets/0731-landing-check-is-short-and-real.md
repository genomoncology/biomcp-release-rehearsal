# 0731 — Landing check is short and real

Status: COMPLETE.
Landed: e4d33a318cf28a3e3b03e752e0e4629bf895f837

Milestone: 5

## Outcome

`make landing-gate` runs cheap checks first, then rights, licensed-content and secret checks, then the outside-in CLI and MCP cases, and stays well under 30 minutes. The full suite is one named command for final release. The coverage floor no longer gates anything, and the mutation run is left out of everything except release, where it can fail or is removed.

## Why

Ian, 2026-11-06: "until we're doing a final release, we don't need to. The final release is the thing that needs the full thing." and "You just need to know that the CLI works." The gate took about 23 minutes, 8 of them in the coverage stage. Survivors pass the mutation run, and the floor sits 4.7 points below measured.

## Proof

- Proof: time the new gate once, against the old stage times, on one commit. Show the cases still run with coverage instrumentation off and that a deliberately failing case fails the gate. Do this first, because every other batch lands through it.

## Starts from

- Starts from: `scripts/landing_gate.py`, `scripts/coverage.sh`, `coverage.floor`, `Makefile`, `release/`.

## Keeps

- Keeps: the same case suite, `audit`, `secrets`, and the rights and licensed-content checks. Coverage stays available as a report.

## Defers

- Defers: nothing.

## Changes

- Changes: `Makefile`, `scripts/`, `release/`, `docs/development.md`.
