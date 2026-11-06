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

## Amendment, 2026-11-06

Ian ruled on the ten studies with purpose limits (m28975): "A is fine thanks", meaning exclude all ten at `restricted`. They are the five TARGET studies limited to pediatric research (`all_phase2_target_2018_pub`, `aml_target_2018_pub`, `nbl_target_2018_pub`, `wt_target_2018_pub`, `rt_target_2018_pub`) and five research-only studies that bar clinical decision-making (`angs_painter_2020`, `angs_painter_2025`, `brca_mbcproject_wagle_2017`, `brca_mbcproject_2022`, `mpcproject_broad_2021`). A stated purpose limit sets the flag, the same as explicit noncommercial wording. Local and `academic` use keep them.

From the ticket review, `records/2026-11-06-ticket-review.md`:

A study left out by policy is not a partial answer: the reply stays complete and carries the notice. Assert the status in the declared use test.

At hosted validation, compare the hosted snapshot's study ids against the reviewed list once, read-only. Ids missing from the review are reported, not guessed.

Per-ticket Makefile goals do not land.
