# 0711 — Code docs build without broken links

Status: COMPLETE.
Landed: ca195c8217ee6eefe7001d7184e2f2d43f8f5c84

Milestone: 5

## Outcome

`cargo doc` builds the whole workspace with no warnings, and the routine gate keeps it that way.

## Why

Row W41 of `records/2026-11-06-pre-release-review.md`, first rejected as hygiene. Ian, 2026-11-06: "if there are any cosmetic issues that we can get done, prioritize them and work on them."

## Proof

- Proof: the failing `cargo doc` run, then a routine gate step that fails on a broken doc link.

## Starts from

- Starts from: the doc comments `cargo doc` names, and `scripts/`.

## Keeps

- Keeps: no published crates.

## Defers

- Defers: nothing.

## Changes

- Changes: doc comments in `crates/` and the routine gate.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Many of the reported errors come from one place. The `singleton!` macro at `crates/biomcp-types/src/response/error.rs:13` uses each usage string as its rustdoc text as well as its wire value and schema enum. Fix the macro with a fixed doc line or a code span. Do not change the strings, which are wire values and asserted output.

Doc comments on schema types also feed schema descriptions, so `current-schema-check` and `schema-release-check` must pass after the fix. The first run stopped after two crates, so later crates may hold more errors than the roughly 30 reported.
