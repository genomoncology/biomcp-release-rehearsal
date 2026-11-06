# 0716 — Wheel ships one copy of the binary

Status: WITHDRAWN. The one-copy route can only be proved on hosted Windows and Intel and could break the first registry entry. The wheel keeps both copies for 1.0, about 16 MB per wheel.

Milestone: 5

## Outcome

The PyPI wheel holds the program once, and both `biomcp` and `uvx biomcp-cli` still work on every platform.

## Why

Row W43 of `records/2026-11-06-pre-release-review.md`, first deferred. The second copy adds about 16 MB compressed. `server.json` runs the package through `uvx`, which needs a `biomcp-cli` command.

## Proof

- Proof: first check whether the MCP Registry entry can run `biomcp` from package `biomcp-cli`, which needs no launcher. Then an installed-wheel case on each release platform for both commands. If the only route is a new launcher that adds more risk than 16 MB is worth, report that to the PM.

## Starts from

- Starts from: `release/artifacts.py`, `server.json`.

## Keeps

- Keeps: the package name `biomcp-cli` and the installed `biomcp` command.

## Defers

- Defers: nothing.

## Changes

- Changes: `release/`, `server.json`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Found while sizing this ticket. The published 0.9.3 wheels hold only `biomcp`, and the public MCP Registry holds no BioMCP entry yet, so the second copy serves only the new `uvx biomcp-cli serve` route in `server.json`. The registry schema has no entry-point field. The one-copy route is `uvx --from biomcp-cli@VERSION biomcp serve`, written with a positional `--from` in `runtimeArguments`. The schema does not define the order in which clients join those parts, and it can only be proved on hosted Windows and Intel runs.

Decision: the wheel keeps both copies for 1.0, at a cost of about 16 MB per wheel. Close this ticket without a change, and reopen the one-copy route after 1.0 if the join order holds for a second client.
