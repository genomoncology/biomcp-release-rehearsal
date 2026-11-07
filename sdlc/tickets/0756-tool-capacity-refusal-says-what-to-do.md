# 0756 — Tool capacity refusal says what to do

Status: COMPLETE.
Landed: 7a2d2c737592aad027ca9cd4c37637891efe8ff1

Milestone: 5

## Outcome

An MCP client that sends many calls at once gets either an answer or a refusal that says how many calls may run at once and when to retry. The limit is configurable, with a stated reason for its default.

## Why

Bench, 2026-11-07: `biomcp-mcp` sent many calls at once, and the server refused 16 with "Tool capacity exceeded". The run still scored 8 of 8, but agents do this often, and each refusal is a wasted call.

## Starts from

- Starts from: where the server counts active tool calls and builds that refusal.

## Keeps

- Keeps: hosted's per-caller fairness and its limits; a refusal stays a refusal when the server is truly full.

## Changes

- Changes: the refusal carries the limit and a retry hint. Decide whether a short wait for a free slot, inside the call's time budget, beats refusing at once, and do the simpler. Check the default against what one agent does.

## Proof

- Proof: a case sends more calls at once than the limit and shows every call either answers or gets the new refusal text, through stdio and HTTP.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: bench mail, 2026-11-07.

## Progress
