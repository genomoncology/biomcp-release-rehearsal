# 0520 — Run the Python checks in a gate and delete dead scripts

Status: COMPLETE.
Landed: 4a19ca247b8a65e3f3d384ed479afbf885d2ff90

Milestone: 5

## Outcome

`scripts/test_schema_release.py` and `scripts/test_reply_diff.py` run in a Makefile target that the landing gate or `docs-check` already runs, and the one-off capture and replay scripts that nothing references are deleted.

- Why: Jarvis's 1.0 code review (m17823, finding 7). `sdlc/planning/architecture.md:31` names `test_schema_release.py` as a check, but no Makefile target, release script or workflow runs it or the 499-line reply diff test. About 840 lines of capture and replay scripts, such as `scripts/author_native.py`, `scripts/gwas_get_replay.py` and the `scripts/protein_*_native.py` files, are referenced nowhere.
- Starts from: the Makefile, `release/`, `.github/workflows/`, `sdlc/planning/architecture.md` and `scripts/`.
- Rules: run the two tests where they fit with no new framework. A test that fails on current main is fixed if the fix is mechanical, or else reported to the PM before deletion. Delete a script only when a search of the repo, the Makefile, release scripts, workflows, tickets' Proof bullets and docs finds no reference. List each deleted file in the review. Evidence on machines is untouched. Do not touch `crates/biodata`.
- Proof: the chosen target runs both tests and fails when either test's assertion is broken on purpose; `rg` shows no reference to any deleted script; the landing gate passes.
- Changes: the Makefile, the two tests if needed, `sdlc/planning/architecture.md` if its citation moves, and deleted scripts.
- Defers: nothing.
