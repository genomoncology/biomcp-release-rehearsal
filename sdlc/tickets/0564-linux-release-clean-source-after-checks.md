# Linux release starts from a clean source tree after the checks

Status: COMPLETE.
Landed: 8b556700912a3e731ae085d5dd427dd06b891864
Milestone: 5

## Outcome

On the Linux x64 release job, `make fmt lint test docs-check` leaves the checkout clean, so `make release-linux` passes the source revision guard without an explicit revision, and the job has disk room for the container build.

- Why: private rehearsal run 41358845085 at ab41e914f passed fmt, lint, test and docs-check on Linux x64, then `make release-linux` refused with "dirty source tree: provide an explicit revision for a local rehearsal" (`release/source_revision.py:12-14`; raw log on Yellow at `biomcp-redux-evidence/0294/private-rehearsal-41358845085/linux-x64-job.log` lines 4466-4478). The log does not name the dirty path. The same log shows the disk at 100 percent with 255M free (lines 4463-4465).
- Starts from: `.github/workflows/release.yml:110-130`, `Makefile` targets fmt, lint, test, docs-check and release-linux, `release/source_revision.py`, `release/linux.sh`.
- Rules: start with a bounded diagnosis: reproduce the check sequence on a clean Linux checkout and name every file it adds or changes outside ignored paths. Fix the cause: a check that writes into tracked or unignored paths writes to an ignored or temporary path instead, or a generated path that should be ignored becomes ignored. Keep the revision guard as it is, and never pass an explicit revision or reset the tree to get past it. Print a bounded `git status --porcelain` in the guard's error so the next failure names its path. If the disk use comes from generated build state the checks leave behind, free it in the same recipe, for example by removing the check build output the release build does not reuse; otherwise record the measured use and leave disk to its own ticket.
- Proof: a test where the guard's error names the dirty path; a Linux run of the check sequence followed by `git status --porcelain` that is empty; the next private rehearsal confirms Linux x64.
- Changes: the checks' output paths, `.gitignore` if needed, the guard's message, and the workflow's cleanup step only.
- Defers: the installed runner fixes owned by the iCite helper and Linux evidence path tickets.
