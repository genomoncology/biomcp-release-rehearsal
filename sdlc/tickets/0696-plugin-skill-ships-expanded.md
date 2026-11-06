# 0696 — Plugin skill ships expanded

Status: OPEN.

Milestone: 5

## Outcome

The plugin's skill guide ships with its front map expanded, with no raw `{{front-map}}` marker, and a client finds `./skills/`.

## Why

Row W30 of `records/2026-11-06-pre-release-review.md`: `skills/SKILL.md:6` is copied raw at `release/workflow.py:119-126`, so every plugin user gets the marker. Traced in code, not run in a client.

## Proof

- Proof: a failing assertion in 0640's plugin section on the built plugin, then the fix.

## Starts from

- Starts from: `release/workflow.py:119-126`, `skills/SKILL.md`.

## Keeps

- Keeps: plugin contents otherwise.

## Defers

- Defers: nothing.

## Changes

- Changes: `release/workflow.py`, its test.
