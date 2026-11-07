# 0761 — CLI accepts entity-first order

Status: COMPLETE.
Landed: be7c3799d364a374a960f1d4a647403862a01e62

Milestone: 5

## Outcome

`biomcp gene get BRAF` runs the same command as `biomcp get gene BRAF`, with the same output and exit code, for every registry command whose order today is verb-first. Today's order stays the documented main form. Help and docs do not change, except one line saying the other order is accepted.

## Why

Ian, 2026-11-07: "Yeah, go ahead and support both subcommands. That lets the agent not make as many mistakes." In the CLI design review run, entity-first typed today failed with an unknown-command error, and one run gave up and used curl (`records/2026-11-07-cli-design-review.md`, option B).

## Proof

- Proof: a case runs each verb-first registry command in both orders and shows identical output and exit code. A case shows that an unknown entity-first word still gets today's unknown-command error. Both fail on main today.

## Starts from

- Starts from: the CLI argument parsing and the command registry. The review's shim mapped the new order onto the build before parsing. Prefer one rewrite of the argument list before parsing over duplicating subcommands.

## Keeps

- Keeps: every current spelling, output, exit code and help text; the 19 hosted MCP tool names and their arguments; the commands that are already entity-first, such as `article citations X`. Where a rewrite would be ambiguous, today's meaning wins.

## Defers

- Defers: making entity-first the documented form, and settling section and facet names, go to the 1.1 MCP entity tools design.

## Changes

- Changes: CLI parsing, one help line, and cases.

## Depends on

- Depends on: nothing. It lands before the hosted retest commit is named.
