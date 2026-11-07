# 0752 — Label section refusal lists the sections

Status: READY.

Milestone: 5

## Outcome

`get drug NAME label` with no label section gives an agent what it needs: the error names the real section choices, or the reply defaults to a useful label answer. The same holds in every mode, MCP included.

## Why

Bench report, 2026-11-07: `biomcp get drug NAME label` is refused and the error lists 'Bounded nonempty text' as the valid values, so an agent cannot learn the section names. It counts against the MCP modes and is the last thing blocking the bench's 100 runs.

## Starts from

- Starts from: the `get drug` grammar and the usage error's `valid_values`.

## Keeps

- Keeps: tool names, schema names and hosted refusals. Fix the general cause: any argument with known choices lists them, not its type description.

## Changes

- Changes: the usage error for a missing or bad section value lists the real values, or a bare `label` lists the sections as its reply. Choose the simpler.

## Proof

- Proof: a case runs `get drug NAME label` through the CLI and through MCP and shows the section names.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: bench mail, forwarded 2026-11-07.

## Progress
