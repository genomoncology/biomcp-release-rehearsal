# 0679 — Typed tool descriptions stand alone

Status: OPEN.

Milestone: 5

## Outcome

Every typed tool's description, and every parameter description including `query` and `input`, is complete without another call: no 'See biomcp_help' in place of content, and no text cut off mid-word. `biomcp_help` gives every flag a description.

## Why

Claude's retest of the live typed server on 2026-11-06 (pin 1bb97e64; hosting m27741 via Jarvis m27744; evidence in `data/biomcp-hosting-evidence/2026-11-05-claude-eval-v2/`): descriptions said 'See biomcp_help', some were cut mid-word, and `biomcp_help`'s flag descriptions were empty. Quickest win named by hosting. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Investigate first: reproduce on current main in typed mode (the retest ran an older pin). If main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Then a structural test over tools/list in typed mode fails on any empty description, any description that only points elsewhere, and any truncation (a description ending inside a word). Routine gate.

## Starts from

- Starts from: the typed tool builder in `crates/biomcp/src/mcp/typed.rs`, 0625's shortened descriptions, and 0653's single description source.

## Keeps

- Keeps: one description source for CLI help, the single tool and typed tools (0653). Descriptions stay short enough for 0683's size budget.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp/src/mcp/`, the schema description source, `tests/`.
