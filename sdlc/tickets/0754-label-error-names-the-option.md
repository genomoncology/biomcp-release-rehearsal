# 0754 — Label error names the option to use

Status: READY.

Milestone: 5

## Outcome

When `get drug NAME label` needs a section, the error's message says to add `--label-section NAME` and gives one full example, so an agent gets it right on its next call.

## Why

Bench, 2026-11-07: an agent spent three failed calls here, each resending the 172 names. The list alone does not say which option takes them.

## Starts from

- Starts from: `crates/biomcp/src/grammar/validation.rs` and 0752's case `drug-label/bare-label-lists-headings`.

## Keeps

- Keeps: the listed names in `valid_values`, tool names and schemas.

## Changes

- Changes: the usage message names the option with an example. A general rule for missing options, not a label special case, if that is as simple.

## Proof

- Proof: the case checks the message through the CLI and MCP.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: bench mail, forwarded 2026-11-07.

## Progress
