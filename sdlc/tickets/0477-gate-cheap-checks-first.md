# 0477 — Run the dependency and secret checks before coverage in the landing gate

Status: COMPLETE.
Landed: 2eb027687d13ea7dd91aaa939ac5ca0d3ebb71f3

Milestone: 5

## Outcome

`make landing-gate` runs `audit` and `secrets` before `coverage`, so a dependency advisory or a secret finding stops the gate in minutes instead of after the hour-long coverage run.

- Why: a read-only audit of the 393 failed jobs since j20000 (m14856) found 9 jobs, about 9 booked core-hours, that passed coverage and then failed at the security checks. Coverage failures cost another 33.6 booked core-hours, and cheap rejections 26.5.
- Starts from: the `landing-gate` prerequisite order in `Makefile` and the existing gate-order test, if one exists.
- Rules: change only the order: `workflow-lint plain-cli-build fmt lint docs-check tensor-proto-check audit secrets coverage`. Each check still runs exactly once. Add no new checker. The gate's passing job is still the one configured command, so landing rules do not change.
- Proof: a test or recorded dry run (`make -n landing-gate`) shows `audit` and `secrets` before `coverage`; a planted secret in a scratch branch fails the gate before coverage starts.
- Changes: the Makefile and its order test.
- Defers: reusing a completed proof across reruns, which belongs to the pm tool.
