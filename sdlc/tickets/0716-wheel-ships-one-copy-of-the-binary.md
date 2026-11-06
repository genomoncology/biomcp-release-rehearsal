# 0716 — Wheel ships one copy of the binary

Status: OPEN.

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
