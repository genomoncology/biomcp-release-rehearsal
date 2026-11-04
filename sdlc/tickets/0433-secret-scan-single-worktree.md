# 0433 — Scan only this checkout's history

Status: COMPLETE.
Landed: 7e347189493e744853912dd4cb046c1c581619c6

Milestone: 5

## Outcome

`release/secrets.py` enumerates history objects and exception reachability with `git rev-list --objects --single-worktree --all HEAD`, so it scans every named ref plus the candidate HEAD and never the detached HEADs of other linked worktrees that share the repository. Replacement objects stay off, and no allowance changes.

- Why: 0407's landing job j20032 passed every product stage, then the history scan flagged 42 blobs reachable only through another job's detached worktree HEAD (0414 at 97a7c826) in the shared repository (coordinator m12883, 2026-10-28). Git includes other worktrees' HEADs in `--all`, which makes the scan depend on unrelated concurrent jobs.
- Starts from: `release/secrets.py`, its tests and 0404's isolated environment.
- Rules: PM ruling 2026-10-28: the scan's scope is this checkout's named refs plus its HEAD. No suppression, no new allowance, no history rewrite.
- Proof: an outside-in case with two linked worktrees shows a secret planted only in the sibling's detached HEAD is not scanned, while secrets planted in the candidate history and in an ordinary named ref still fail.
- Changes: `release/secrets.py` and its cases.
- Defers: nothing. Priority: it blocks landing 0407 and every concurrent batch.
