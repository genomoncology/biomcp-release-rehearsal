# 0763 — MCP offers one typed tool per entity

Status: COMPLETE.
Landed: 1f8f2165fc8b341d402717ae6c49e1ee3779b99a

Milestone: 5

## Outcome

The typed MCP mode lists the 17 tools in `records/2026-11-07-entity-tool-map.md`: 13 entity tools plus `biomcp`, `help`, `search` and `fetch`. Each entity tool takes `action` and one typed object named after the action. All 58 registry commands are reachable through a typed schema, and nothing takes a free-form object. `biomcp_reference` and today's other typed tool names are gone. Hosted mode serves the same 17 tools, with hidden sources filtered inside each action's schema.

## Why

Ian, 2026-11-07: "I need that BioMCP 1.0 done with all the bug fixes and the new tool design before I can change hosting" (`decisions/2026-11-07-entity-tools-in-1-0.md`). In the tool map runs, first calls named the right tool and action 40 times against 7 for DeepSeek, and failed calls fell from 92 to 32.

## Proof

- Proof: one case calls every tool and action once, through stdio and hosted HTTP, offline where the existing cases are, and shows each call reaches its command and returns that command's reply. A coverage case shows every registry command is reachable from exactly the placements in the map, and fails if a command is unplaced or any schema holds a free-form object. The existing replay and tool tests pass with their tool names updated.

## Starts from

- Starts from: the record's map table, the machine-readable map in the study data, and `crates/biomcp/src/mcp/typed/` (catalog, grouping, arguments, description, followups, help, summary). Next-call hints, argument recovery and refusal text must name the new tools and actions. Generate the list from the registry; do not hand-write 63 schemas.

## Keeps

- Keeps: each command's own schema inside its action, every reply, `biomcp` taking the CLI argument array, `search` and `fetch` for ChatGPT, hosted fairness, call limits and the 0756 capacity refusal, the `legacy` and `current` modes as they are, and the CLI.

## Defers

- Defers: nothing. Docs, the plugin skill and release notes are 0764.

## Changes

- Changes: `crates/biomcp/src/mcp/typed/`, `typed.rs`, `protocol.rs` and `http/call.rs` where they name tools, `crates/biomcp/typed_front_map.rs`, the typed and hosted tests under `crates/biomcp/tests/runner/`, and the generated schemas.

## Depends on

- Depends on: nothing.
