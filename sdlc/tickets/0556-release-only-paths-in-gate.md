# 0556 — Run release-only paths in the routine gate

Status: OPEN.

Milestone: 5

## Outcome

The routine gate exercises the release build command and the unprepared install-gate preparation that the release workflow runs, and the cheap check parses every tracked `#!/bin/sh` script with a native `/bin/sh -n`. A release-only path no longer meets its first test on a hosted runner.

- Why: another team's root cause analysis of three failed hosted release runs found that a Linux-only private gate let release-workflow-only code and macOS `/bin/sh` parsing reach hosted runners untested (Jarvis m19189). The coordinator's check (m19193) found two of the same gaps here. `release.yml:116-117` runs the artifact build and unprepared install gate, but `release/test_gate.py:102` passes `--prepared` and skips `release/gate.py:112-120`, and the successful build test `release/test_artifacts.py:13` runs only from `release/matrix.sh:21`. No gate stage in `scripts/landing_gate.py:6-7` parses `#!/bin/sh` scripts, and `matrix.sh` and the diagnostic scripts have no parse coverage. Temp path aliases are already covered.
- Starts from: `release/test_gate.py`, `release/gate.py`, `release/test_artifacts.py`, the Makefile's release test goal, the cheap check from 0552 and `scripts/landing_gate.py`.
- Rules: add fixture-backed tests of the real build command and the unprepared gate preparation to the existing release suite, without compiling or installing packages; real compilation and installation stay in release qualification. The cheap check runs `/bin/sh -n` on every tracked file whose first line is `#!/bin/sh`, and the macOS machine runs it once in this ticket's proof. Add no new machine, job or framework.
- Proof: a test fails when the unprepared preparation path breaks; a deliberately broken `#!/bin/sh` script fails the cheap check on Linux and on the macOS machine; the routine gate passes on main.
- Changes: the release tests, the Makefile and the cheap check.
- Defers: none.
