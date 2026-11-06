# 0688 — Pre-release whole-project review of 2026-11-06

Status: COMPLETE. Three reviews and the weighing are in records/2026-11-06-pre-release-review.md; every finding ruled and ticketed

Milestone: 5

## Outcome

Three fresh reviewers review main f33f42a55 as one system, read-only, following the adversarial project review skill: one for correctness, stored state, reliability, operability and performance; one for security, data rights and dependencies; one for release, gates, tests, maintainability, docs, plan and tickets. A fresh weigher weighs each finding. The PM rules on each and tickets the accepted ones with their evidence, and the merged report is kept as `records/2026-11-06-pre-release-review.md`.

## Why

Since the last whole-project review (2026-10-28), 1.0 grew: typed MCP tools, the hosted evaluation fixes, release-section tooling, and repeated late finds in release and gate runs (`records/2026-11-05-release-finds-one-at-a-time.md`, `records/2026-11-05-timing-tests-under-load.md`). The skill asks for a review before a release.

## Proof

- Proof: the merged report with a verdict and a before-release table, the weighing, and one ticket per accepted finding.

## Starts from

- Starts from: main f33f42a55, `~/.claude/skills/adversarial-project-review/`, and the earlier reviews in `records/`.

## Keeps

- Keeps: everything; the review is read-only and builds only under this ticket's key.

## Defers

- Defers: fixes, to the tickets the PM files.

## Changes

- Changes: `sdlc/records/` only.
