# 0710 — Restricted mode honors cBioPortal study flags

Status: OPEN.

Milestone: 5

## Outcome

Each cBioPortal study can carry a commercial-use flag. At `restricted`, a flagged study is left out of study search, study reads and recurrence counts, and the reply says a study was left out. `commercial` and `academic` keep today's behavior. The flag list is checked in, reviewed, and refreshed when the study list changes.

## Why

Ian, 2026-11-06, on the record's Q2: "I certainly want to support the idea that the restricted mode understands the studies at the flag level. That seems like a really good setting." cBioPortal's terms are ODbL "unless otherwise noted", and some studies restrict commercial use in their study information. Row W10 of `records/2026-11-06-pre-release-review.md`.

## Proof

- Proof: recorded-reply cases at `restricted`, failing first: a flagged study is absent from search, a direct read names why, and recurrence totals exclude it. The same cases at `academic` still return it. One read of the public study list records where the restriction wording lives and which studies carry it, with the exact wording for each. Explicit noncommercial wording sets the flag. Unclear wording goes to Ian; agents do not judge terms. Routine gate.

## Starts from

- Starts from: `crates/biomcp-adapters/src/cbioportal/mod.rs`, `hosted_use.rs`, `response/recurrence.rs`, `variant/recurrence.rs`, and `decisions/2026-11-04-hosted-source-modes.md`.

## Keeps

- Keeps: cBioPortal on in every mode, and its `attribution` label. Hosting's startup check, a `search study` TCGA lookup that needs at least one result, keeps passing; if it cannot, tell hosting its replacement first.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/src/cbioportal/`, `crates/biomcp-query/`, `docs/`, `tests/`. BioMCP applies the flag when it reads a study, so hosting's existing snapshot stays readable and needs no refetch.
