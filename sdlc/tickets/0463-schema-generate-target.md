# 0463 — Generate schemas through one checked target

Status: COMPLETE.
Landed: 15bd171ec9799120fb8dfd7d5311c40174d6e1e2

Milestone: 5

## Outcome

`make schema-generate` regenerates the checked-in schemas through a generation mode in `scripts/schema_release.py`, and fails unless its one named generation test ran and passed. Compatibility is still checked before any file is written.

- Why: three jobs (j20033, j20283, j20389) selected zero tests because the feature flag removed the test target, and reported success with no schemas generated.
- Starts from: `scripts/schema_release.py`, the Makefile and the schema generation test.
- Rules: reuse the script's existing checking recipe; no second generator.
- Proof: a run where the feature flag hides the test fails with a named message; a normal run regenerates byte-identical schemas on unchanged code.
- Changes: the script, the Makefile and `docs/development.md`.
- Defers: nothing.
