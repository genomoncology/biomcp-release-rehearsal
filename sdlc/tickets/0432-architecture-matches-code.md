# 0432 — Make the architecture describe the code

Status: COMPLETE.
Landed: 63d0cb28eeffa9387023901d5ea827558fbbbd42

Milestone: 5

Depends on: 0428, 0427, 0456, 0457, 0484, 0485

## Outcome

`sdlc/planning/architecture.md` and the decisions it cites describe the shipped layering, command table, source declarations, value rule, transport call, error types, status rules and test strategy, and name the check that enforces each rule. Stale sections go. The ticket decides whether `scripts/reply_diff.py` stays (ruling 8).

- Why: the code quality review found architecture text that the code contradicts, and the restructure changes most sections.
- Starts from: `sdlc/planning/architecture.md`, `decisions/2026-10-23-crates-enforce-dependency-direction.md`, `sdlc/features.md`, `docs/development.md`, `scripts/reply_diff.py`.
- Rules: each rule names its check. A rule with no check gets one or goes.
- Proof: the docs check fails on a planted architecture rule that names a missing check file; a reviewer who did not work on the restructure confirms each section against the code in a recorded read.
- Changes: the files above.
- Defers: nothing.
