# 0680 — Typed follow-ups are typed calls with sections in their own field

Status: OPEN.

Milestone: 5

## Outcome

In typed mode, every follow-up suggestion is a typed tool call. No command-line follow-up appears beside it, so no `--config` or server path can leak. A suggestion that opens a section puts the section name in the `section` parameter, never inside `input`.

## Why

Claude's retest of the live typed server on 2026-11-06 (pin 1bb97e64; hosting m27741 via Jarvis m27744; evidence in `data/biomcp-hosting-evidence/2026-11-05-claude-eval-v2/`): command-line follow-ups with `--config /etc/biomcp/config.json` still appeared next to typed ones, and typed suggestions put section names inside `input`. Quickest wins named by hosting. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Investigate first: reproduce on current main in typed mode (the retest ran an older pin). If main already does what the Outcome says, record the evidence and close the ticket. Fix the general cause and tune nothing to the example. Then replayed typed cases assert each follow-up names a tool in tools/list, validates against that tool's input schema, carries no CLI string, and uses `section` for sections. Routine gate.

## Starts from

- Starts from: the `next` builder, 0654's hosted projection, and the typed continuation code in `crates/biomcp/src/mcp/`.

## Keeps

- Keeps: single-tool and CLI follow-ups unchanged.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp/src/mcp/`, the `next` builder, `tests/`.
