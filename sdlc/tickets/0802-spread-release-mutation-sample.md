# 0802 — Spread the release mutation sample across changed code

Status: COMPLETE.
Landed: ccb358d7f974246f909bd234b9e66cdada5611d6

Milestone: 5

## Outcome

The existing bounded mutation planner selects deterministically across eligible changed Rust files, prioritizing mutants located on changed current-side lines. Its receipt exposes representation and omissions without promising semantic coverage.

## Why

At candidate 34d6e02d, calibration j36890 selected all 20 mutants from unchanged early CT.gov parameter methods, missing the new pagination accounting and every other changed file. An independent read-only review confirmed this Medium selection-usefulness defect; truthful collection alone does not meet 0788's changed-behavior sampling goal.

## Proof

- Extend the existing fake native inventory test with a large first changed file whose changed-line mutant comes late, two other changed files and a rule-only file. Require changed-line priority, file representation when the calculated limit permits, deterministic unique selection, unchanged count limits and complete unrun accounting.
- Retain exact native discovery/selection and collection checks. Cover changed files without eligible mutants, insufficient selection slots, deletions with no overlapping mutant and rule-only fallback.
- Run existing focused release mutation tooling checks and routine gate. Long product mutation runs remain delegated under 0788 after landing.

## Starts from

- release/mutations.py sorts by changed-file order and takes names[:limit] at selected source 34d6e02d1ebeac4d55e44e05099854f15e718d57. Actual previous candidate remains 0adbae81319cbfd89066c880ced85391f7a4775a. j36890 inventory has 1,693 mutants; its 20 selected CT.gov lines 64–130 precede changed received-count handling at lines 192 and 218. Preserve j36891 as cost-only calibration evidence.

## Keeps

- Existing previous/candidate comparison, complete native inventory, exclusions, count calculation, deadlines, no-shuffle execution, exact selection checks, scores and unrun accounting. No outcome-based selection, widened budget, new dependencies/framework, runtime changes or survivor reclassification.
- Use current-side changed-line ranges from the same Git diff. Within each changed-file queue put overlapping native mutant locations first, retaining stable order inside both groups; choose one per file per round. Consume rule-only scope after changed-file queues are exhausted. Record selected counts per file, overlap counts and unrepresented/no-eligible changed files. File/line coverage is a mechanical approximation, not proof of every behavior.

## Defers

- The actual final bounded sample and its independent outcome assessment remain in 0788. Only the affected mutation section repeats with the real tooling revision; selected product 34d remains immutable. Hosting, docs and Python diagnostics lie outside core Rust mutation scope.

## Changes

- release/mutations.py, its existing tests and necessary selection-description documentation.
