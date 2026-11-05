# 0668 — Search FDA device approvals by name or company

Status: OPEN.

Milestone: 5

## Outcome

`search diagnostic` searches FDA premarket approvals (PMA) and 510(k) clearances by device name, trade name or company, in single and typed mode, and each hit gives the ID that `get diagnostic` accepts. Help and the typed tool schema list both sources for search, not only for get.

## Why

In the hosted evaluation of 2026-11-05 (hosting m26488), `get diagnostic` listed fda-pma but `search diagnostic` did not, so the agent could not find a device approval (Optune Pax) and fell back to web search. Ian put every finding in 1.0 (`decisions/2026-11-05-everything-in-1-0.md`).

## Proof

- Proof: Investigate first: reproduce on current main in single and typed mode. If main already does what the Outcome says, record the evidence and close the ticket. Otherwise, add replayed cases: a PMA search and a 510(k) search that each return a device hit whose ID `get diagnostic` opens, plus a no-match case that returns empty and not an error. Run the routine gate. Fix the general cause and tune nothing to the example device.

## Starts from

- Starts from: `crates/biomcp-query/src/diagnostic/device_search.rs` (already calls the `fda_pma` and `fda_510k` adapters), `crates/biomcp-query/src/diagnostic/pma.rs`, the diagnostic command's source list, and the typed diagnostic tool schema.

## Keeps

- Keeps: existing diagnostic sources and their results, and openFDA device endpoints only (public, no key needed). Hosted mode serves these sources only if its allowlist already permits openFDA.

## Defers

- Defers: CMS coverage decisions and other new device data sources, which wait for Ian's ruling on new sources.

## Changes

- Changes: `crates/biomcp-query/src/diagnostic/`, the diagnostic CLI and typed tool wiring, `tests/`.
