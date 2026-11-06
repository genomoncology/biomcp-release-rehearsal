# 0683 — Typed tools fit clients that load every definition

Status: COMPLETE.
Landed: 7a0cab3c7f89fa673ce858821bd3962523bedf57

Milestone: 5

## Outcome

Typed mode's tools/list stays within a stated size budget for clients that load every definition up front, measured in tools and in bytes of definitions. The budget is a named, configurable constant. If today's set of about 58 tools exceeds it, the least-used entities are grouped behind one reference tool, as Claude's 2026-11-05 design note suggested, and the escape-hatch `biomcp` tool remains.

## Why

Claude's retest of the live typed server on 2026-11-06 (pin 1bb97e64; hosting m27741 via Jarvis m27744; evidence in `data/biomcp-hosting-evidence/2026-11-05-claude-eval-v2/`): about 58 tools is heavy for clients that load every definition up front. The earlier design note suggested about 15 tools. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Measure current tool count and definition bytes on main and record them. Choose a default budget from the research record (`records/2026-11-04-mcp-skills-and-design-research.md`, tool search pays off past about 10 tools or 10K tokens) and explain it. A test fails when tools/list exceeds the budget. Typed parity cases still pass through the grouped tool. Routine gate.

## Starts from

- Starts from: `crates/biomcp/src/mcp/typed.rs`, the typed parity cases, and the research record.

## Keeps

- Keeps: every capability stays reachable; typed mode stays opt-in for the hosted test server; single-tool mode unchanged.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp/src/mcp/`, typed cases, `tests/`.

## Amendment, 2026-11-06

PM ruling (m27799). Measured on main 6b1618cc2 (j31538): typed mode has 61 tools and 72,769 bytes of compact definitions, hosted typed 59 tools and 71,450 bytes, single mode 3 tools. The grouping follows Claude's 2026-11-05 design note, the only usage evidence we have; both evaluations used adverse events and devices. Direct tools: search and get for drug, trial, article, variant, disease and diagnostic; `get_gene`; `search_adverse_events`; help; research search and fetch; the `biomcp` escape hatch. One reference tool covers pgx, pathway, protein, gwas, cell line and study. Defaults: 20 tools and 48 KiB, both named and configurable. 0679's complete descriptions come before the byte budget. If the total exceeds it, report the measurement to the PM; do not group a clinical entity or truncate a description.
