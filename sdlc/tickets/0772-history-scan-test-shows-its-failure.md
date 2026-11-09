# 0772 — History scan test shows its failure

Status: COMPLETE.
Landed: 1b6f0f6d6c7f5860ca972e20a7457d2e1b082a17

Milestone: 5

## Outcome

`scripts.test_gate_stages.GateStages.test_release_checkout_scans_complete_candidate_without_unrelated_refs` passes on every host and load, and when the history scan fails it shows the safe failure category in the assertion message.

## Why

Practice release run 42464420823 at c33d2c330 failed in the Linux x86_64 `qualify` job, docs-check, `gate-stage-check`: "AssertionError: 2 != 0 : fixture history scan changed" at `scripts/test_gate_stages.py:222`, the first `screen(0)`. Exit 2 is a screening error, not a finding. The same test passed on Linux in run 42451038807 at dee83eff6 and on the M5 at c33d2c330 (j35782), and nothing it reads changed between those commits. The assertion hides the scanner's stdout and stderr, so the cause is not in the log.

## Proof

- Proof: the cause of the exit 2 is named from the code or a reproduction (for example a scanner download, a git command under a missing environment variable, or a timeout), and fixed. The assertion message carries the scanner's safe category output, which never holds a matched value. The test run 20 times in one job, alongside a CPU load, passes every time.

## Starts from

- Starts from: `scripts/test_gate_stages.py` lines 170-230 and `release/secrets.py` `history` and its error categories. 0769 changed this test's token to a fixed synthetic value.

## Keeps

- Keeps: the scanner pin and config, the scope guard checks, and the rule that no matched value reaches output.

## Defers

- Defers: nothing.

## Changes

- Changes: that test, and `release/secrets.py` only if the cause is there.

## Depends on

- Depends on: nothing.
