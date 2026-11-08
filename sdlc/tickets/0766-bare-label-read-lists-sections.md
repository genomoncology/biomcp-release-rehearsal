# 0766 — Bare label read lists sections

Status: COMPLETE.
Landed: 46bf995112f443c3c144e8bddc610078fa13be58
Opened as: 2026-11-08. Release full suite j35400 at 88a6b79e5 failed library-tests: drug_contracts still expects a bare label section to be refused at deserialization.
Opened as: 2026-11-08. Release full suite j35338 at bc44fca0b failed docs-check: the front map's new label line names the flag --label-section, and the front map must name commands only. Fix: name label_section instead.

Milestone: 5

## Outcome

`get drug NAME label` without `--label-section`, and the same call through the drug tool, returns a successful reply that lists the label's section names and says to pass one as `label_section` (`--label-section` on the CLI). It no longer returns a wrong-argument refusal. The plugin skill's command map and the drug tool's description both say that a label read takes a section.

## Why

The agent bench (via Jarvis, 2026-11-07): the bare label read was the largest repeated failed call, with 17 of 43 wrong-argument refusals across 13 runs in round 1. Agents follow the skill's map, which shows `label` with no hint that a section is needed.

## Proof

- Proof: a recorded case runs the bare label read on the CLI and through MCP, and shows exit 0 with the section list and the hint. It fails on main, which refuses.

## Starts from

- Starts from: where the label section is required and refused (`crates/biomcp/src/grammar/validation.rs` and the drug label path), `biomcp_types::OPENFDA_LABEL_HEADINGS`, and the plugin skill's command map.

## Keeps

- Keeps: label reads with a section, unchanged. An unknown section name is still refused, with the list.

## Defers

- Defers: the bench's smaller groups (invented pgx forms, diagnostic filters per source, trial short flags, who-pq as a section). The entity tools' typed schemas (0763) cover the first and the last. The rest go to the next bench review.

## Changes

- Changes: the drug label path, the skill's command map, the drug tool description, and one case.

## Depends on

- Depends on: 0763 for the drug tool description; it stacks on 0763 if it touches the typed descriptions.
