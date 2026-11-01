# 0552 — Report every cheap check failure in one run

Status: OPEN.

Milestone: 5

## Outcome

The cheap pre-job step runs format, workspace Clippy, the tools check and the ratchets as separate goals, finishes all of them after one fails, and then returns failure with every failure printed. Later gate stages still stop at the first failed stage.

- Why: the coordinator's root cause analysis of 14 failed logs on 0550 and 0551 (m18756). The launchers call `make fmt lint ratchets`, so Make stops at the first failed goal, and `lint` stops before the tools check when workspace Clippy fails. A developer fixes one failure, reruns and meets the next.
- Starts from: the Makefile's `fmt`, `lint` and `ratchets` goals, the two launchers that call them, and the cheap step from 0535.
- Rules: split `lint` into callable goals for workspace Clippy and the tools check. One `check-cheap` goal runs the cheap goals sequentially with Make's keep-going and Cargo's `--keep-going`, and exits nonzero if any failed. Route both launchers through it. Add no new checks.
- Proof: a branch with a format error and a Clippy error gets both reported in one run and a nonzero exit; a clean branch passes; a full gate with a failed cheap step runs no later stage.
- Changes: the Makefile and the two launchers.
- Defers: none.
