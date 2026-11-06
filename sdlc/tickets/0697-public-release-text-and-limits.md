# 0697 — Public release text and limits

Status: OPEN.

Milestone: 5

## Outcome

Release notes and `candidate.json` hold no internal wording and state the macOS floor. The Intel Mac time limits rise to 180 minutes in `release.yml:96` and `recover-section.yml:34`. Built wheels pass `twine check`, and the installer byte check runs before the tag. AlphaGenome's Apache-2.0 protos have their notice in `release/notices.py`.

## Why

Rows W9, W29, W34 and W49 of `records/2026-11-06-pre-release-review.md`. Intel qualified at 119m44s against 120 minutes, and more cases have landed since. Publication commands first run at the tag, and PyPI versions cannot be reused.

## Proof

- Proof: a test that fails on the internal wording, `twine check` in a release section, and the notices check naming the protos. Name in 0293 which publish commands stay unproved until the tag.

## Starts from

- Starts from: `release/workflow.py:138,187`, `release/notes.py:22`, `release/notices.py`.

## Keeps

- Keeps: no TestPyPI or scratch registry. Full-suite-once-per-platform waits; 1.0 has no deadline.

## Defers

- Defers: nothing.

## Changes

- Changes: `release/`, `.github/workflows/`.
