# 0725 — Paging never offers an empty page

Status: OPEN.

Milestone: 5

## Outcome

A reply offers a next page only when one exists. When all rows of a known total are returned, no next page appears.

## Why

Item 11: sponsor search offered a next page after 20 of 20; the next page was empty. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a case at exactly the total, one row short, and an unknown total, for each paging family. Works with the end-of-results fix in 0693.

## Starts from

- Starts from: `crates/biomcp-query/src/next.rs` near line 94 and `window.has_more()`.

## Keeps

- Keeps: stored pool behavior.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Reproduced live: a search with exactly 20 results offers a next page, and that page is empty. `trial.rs` `apply_page` offers the next page on the source's page token alone and never compares the known total. A recorded ClinicalTrials.gov reply shows a token on a full last page. Other known-total families already check the total.

The Outcome's promise covers known totals. Where a source gives no total, a full window may still offer a next page, because no command fetches an extra page to find out. That page comes back empty and says so.

Fix `apply_page`, keeping the token when the total is unknown. Do not add a global guard in `next::finish`, because a reply's page can describe an outer list while a step pages a section.

The Proof's cases run per mechanism, not per family: a source token, a known-total window, a stored pool and an unknown total. Each has a case at exactly the total and one row short. Land after 0717, and include a drug name that expands.
