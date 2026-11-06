# 0711 — Code docs build without broken links

Status: OPEN.

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
