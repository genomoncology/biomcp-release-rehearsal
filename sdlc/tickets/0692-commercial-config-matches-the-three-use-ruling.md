# 0692 — Commercial config matches the three-use ruling

Status: COMPLETE.
Landed: 76b744f0f1c287e0675e85aa5b7a8278eed5569f

Milestone: 5

## Outcome

The README's commercial setup leaves free-for-commercial sources on, and its own examples run. The retired `restricted` alias is gone, and `licensed` is a usable disable group. The README's hosted paragraph links to `docs/configuration.md` and restates nothing. Registry terms text and `unverified` flags match the terms review (m22678), and Cancer Hotspots is `attribution`.

## Why

Rows W3, W4, W10, W38, W46, W50, W51, W53 and W56 of `records/2026-11-06-pre-release-review.md`. The README config turns off 25 of 60 sources including PubMed, ClinicalTrials.gov and ClinVar (reproduced, j32009), against Ian's m22678: "There shouldn't be any limits to anything that's free for commercial use."

## Proof

- Proof: a README case that runs the commercial config and its examples, failing first; a test that `licensed` disables COSMIC and OMIM components. Docs lines: hosted stdio scope, `BIOMCP_SQLITE_DIAGNOSTICS`, patient identifiers in query text, CPIC in `docs/cautions.md:54`, the plain `serve --http` call cap, and the docs items in W38. Routine gate.

## Starts from

- Starts from: `crates/biomcp-*/src/registry/groups.rs`, `registry/legacy.rs`, `README.md:65-67`, `release/notes.md`, and Ian's terms note quoted in m22678.

## Keeps

- Keeps: use labels other than those named. cBioPortal's restricted studies belong to 0710. No word-matching test of terms text.

## Defers

- Defers: nothing.

## Changes

- Changes: the registry, adapter terms strings, `README.md`, `docs/`, `release/notes.md`, `tests/`.
