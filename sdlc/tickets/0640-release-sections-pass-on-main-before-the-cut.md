# 0640 — Release sections pass on main before the cut

Status: OPEN.

Milestone: 5

## Outcome

`release/qualification.md` lists every release section with the one command that runs it alone, its host and its inputs. Before the next candidate is cut, each section has passed alone against a main commit on its real host, so qualification starts from sections already known to work.

- Why: `decisions/2026-11-05-release-finds-no-bugs.md`. Eight of the nine problems found during 1.0 qualification were in sections that had only ever run against a cut candidate (`records/2026-11-05-release-finds-one-at-a-time.md`).
- Starts from: `release/qualification.md`; the Makefile goals `release-cold`, `release-mutations`, `release-linux`, `secrets-full` and `schema-release-check`; `release/published.py` (`legacy-installer`, `plugin`); `scripts/check_windows_release.sh`; `release/smoke.py`; and the section list in 0293.
- Rules: reuse the existing commands, with no new launcher, framework, checker or workflow. A section that needs hosted artifacts, such as the Windows VM wrapper, takes the newest retained hosted artifacts and names their product commit. The hosted public rehearsal stays a manual dispatch under `decisions/2026-11-02-public-rehearsals.md`. When a section fails, only that section reruns.
- Proof: a preflight at one main commit records each section's own pm job and exit, in the order the document gives, and every section passes. The next candidate cut cites that preflight.
- Changes: `release/qualification.md`; the section list, commands and order.
- Defers: a scheduled preflight. It runs by hand before each cut until a second release shows whether a schedule pays.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

The document on this ticket's branch is stale. 0666 already made the recovery action take a main revision, and its candidate lines are superseded. Refresh it, take machine names out of it, and land the document now: only the preflight runs are held.

Stage, the staged artifact scan, attestation and staged Homebrew are sections too, and the document lists them. Rows change as 0696, 0697, 0712, 0713 and 0716 land.

Tooling tickets land on proof from a local real host. The preflight is their Windows and Intel proof, and a failure there reopens the ticket that caused it.
