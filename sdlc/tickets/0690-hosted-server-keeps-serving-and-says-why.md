# 0690 — Hosted server keeps serving and says why

Status: OPEN.

Milestone: 5

## Outcome

One failed accept or thread start sheds that request, logs why, and leaves the server running. Each call logs one line with tool, duration, status and failed source names, and no arguments. Framing readers rise to the existing worker bound, and the docs state what `/health` really promises. Partial answers survive the hosted time limit if a replay test shows they are lost today.

## Why

Rows W17, W18, W22 and W23 of `records/2026-11-06-pre-release-review.md`. `http.rs:132,142,168` exit on one failure and `:112` discards the error (reproduced, harsh setup). Idle sockets reached 503 on a raw port (reproduced twice), though not through our wrapper. The wrapper cannot see partial or failed sources inside a 200 reply, and hosted validation needs that. Ian puts a solid AWS service first.

## Proof

- Proof: a forced-failure test for accept and thread start, a test of the log line that also proves no arguments or keys appear, a `/health` test on a damaged table, and a replay test for W23 first. Routine gate.

## Starts from

- Starts from: `crates/biomcp/src/mcp/http.rs` and the hosted notes in the record.

## Keeps

- Keeps: admission limits and refusals. No separate reader pool. No secret or query text logged.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp/src/mcp/`, `docs/`, `tests/`.
