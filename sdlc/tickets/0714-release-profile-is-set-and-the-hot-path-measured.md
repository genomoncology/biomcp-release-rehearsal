# 0714 — Release profile is set and the hot path measured

Status: OPEN.

Milestone: 5

## Outcome

The workspace states its release build settings, and one recorded measurement shows where a warm and a cold call spend time, including connection setup and store open.

## Why

Row W26 of `records/2026-11-06-pre-release-review.md`, first deferred. `Cargo.toml` has no `[profile.release]`. Ian, 2026-11-06, wants the project "as clean and as free of bugs and issues as possible".

## Proof

- Proof: a recorded-reply timing run before and after, under `pm job`, with numbers in the ticket. Any fix follows the measurement. If nothing dominates, say so and change only the profile.

## Starts from

- Starts from: `Cargo.toml`, the serve path in `crates/biomcp/src/mcp/`.

## Keeps

- Keeps: behavior and binary channels. No live provider load.

## Defers

- Defers: nothing.

## Changes

- Changes: `Cargo.toml`, and code only where the numbers point.
