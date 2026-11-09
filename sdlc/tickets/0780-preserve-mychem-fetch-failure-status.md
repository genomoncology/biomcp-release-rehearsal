# 0780 — preserve-mychem-fetch-failure-status

Status: COMPLETE.
Landed: 07ed6f7bfec6d505f42e9d74c2d8a66fd5a81116

Milestone: 5

## Outcome

A capped MyChem drug-name lookup preserves the selected record fetch's failed, absent or already partial status. Only an otherwise complete selected answer becomes partial because the 250-row name search stopped early.

## Why

The current resolver can relabel a failed record fetch as partial. Idea 0760 explicitly scheduled this before the 1.0 cut; it must be fixed before publication.

## Proof

- Through the CLI's existing drug-name paging fixture, a capped product-only candidate whose selected fetch fails stays failed with the corresponding exit status; a successful capped answer remains partial; an exhausted successful product-only answer remains complete. Show the failure regression on current main before the guard.

## Starts from

- `crates/biomcp-query/src/drug/resolve.rs::get`, `status::item`, `tests/cases/drug/name-get-pages.yaml` and idea 0760. Current main 99e6da6b has the same runtime code as selected b380.

## Keeps

- The 250-row discovery cap, name ranking, resolution annotations, caller budget, source failure details and ordinary status/exit mapping. No extra requests or pagination changes.

## Defers

- The separate over-limit reply diagnostic in idea 0760 is assessed separately; do not silently close it with this fix.

## Changes

- Drug-name resolver and its owning outside-in paging fixture.
