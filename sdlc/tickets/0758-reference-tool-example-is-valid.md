# 0758 — The reference tool's example is a valid call

Status: READY.

Milestone: 5

## Outcome

Every example call in a typed tool's description validates against the schema of the command it names, and a test fails if one does not.

## Why

Entity tools study, 2026-11-07 (`records/2026-11-07-entity-tools-study.md`): `biomcp_reference` shows `{"command":"get_protein","arguments":{"id":"P04637"}}` twice, but `get_protein` takes `input`, not `id`. A model that copies the example makes a failed call.

## Starts from

- Starts from: `crates/biomcp/src/mcp/typed/grouping.rs` lines 103 and 108.

## Keeps

- Keeps: the 19 hosted tool names and their schemas; only description text changes.

## Changes

- Changes: the example uses the command's real arguments, taken from its schema rather than typed by hand where that is simple; a test checks each description example against the named command's input schema.

## Proof

- Proof: the new test fails on today's example and passes on the fixed one.

## Defers

- Defers: the entity tool design, which goes to 1.1 if Ian agrees.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: the entity tools study.

## Progress
