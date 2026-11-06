# 0734 — Staged artifact scan passes on compiled packages

Status: OPEN.

Milestone: 5

## Outcome

The release workflow's staged artifact secret scan finishes clean on the compiled Linux and Windows packages, so a practice release gets past staging to signing and staged Homebrew. Each of the 12 `generic-api-key` findings has a reviewed cause, and the scan still fails on a real key.

## Why

The last practice release passed the candidate and all five native platforms, then stopped at this scan with 12 unclassified findings in compiled artifact metadata, so signing, staged Homebrew and everything after them never ran. No ticket owned it. Ian's goal for tonight is the practice release working on GitHub (2026-11-06).

## Starts from

- Starts from: `release/secrets.py`, the stage job in `.github/workflows/release.yml`, `release/secrets.ignore.jsonl`, 0293's progress note and its retained packages, and 0580 and 0633 for how reviewed findings were handled before.

## Keeps

- Keeps: every scanner, ledger and archive guard. A finding is classified by rule, artifact, offset and fingerprint; no matched value is printed or recorded, and none is waved through without a reviewed cause. A real credential in any finding stops the work and goes to the PM.

## Changes

- Changes: `release/`, `.github/workflows/release.yml`.

## Proof

- Proof: the scan run locally on the retained packages fails before the fix and passes after it; a key injected into a copy still fails. The next practice release reaches attestation and staged Homebrew.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.
