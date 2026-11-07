# 0747 — Pinned snapshots report age, not partial

Status: OPEN.

Milestone: 5

## Outcome

On a hosted pinned snapshot, a bundle past its local freshness date still answers as complete, and the reply states the data's date. Everywhere else, a stale bundle stays as it is today: partial until refreshed.

## Why

Hosting found `get gene BRCA2 clingen` reads stale with status partial, though its content is right. `fresh_until` is a local refresh policy, and a pinned snapshot is never refreshed, so every bundle turns stale as it ages. A longer date would hide age, and hosting's snapshot cannot be refetched in full.

## Starts from

- Starts from: `crates/biomcp-fetch/src/bundle/read.rs`, and the data age shown by 0718.

## Keeps

- Keeps: store format, cache namespace, BioData pin, and stale behavior outside a pinned snapshot. Any startup flag is agreed with hosting first.

## Changes

- Changes: `crates/biomcp-fetch/src/bundle/`, the hosted startup path, and `tests/cases/mcp-hosted/`.

## Proof

- Proof: a hosted case with a bundle past `fresh_until` replies complete with its date, and the same bundle on the CLI stays partial.

## Defers

- Defers: nothing.

## Depends on

- Depends on: 0718.

## Amendment, 2026-11-07

Two findings from the 0718 stack, in this ticket because both concern how age is reported:

- Case budgets depend on the calendar. 0718's "old data" notice is longer past 365 days, so fixtures with a publisher date will fail their byte budgets once that date passes: the cell-line assay case after 2027-05-01, the 2026-10-23 fixtures in October 2027. Pin "today" in those cases. A test must not fail because the date changed.
- Page 2 of the trial gene-notices case has no date notice, while page 1 and count-only do. Decide whether every reply states its date, and make the code match.
