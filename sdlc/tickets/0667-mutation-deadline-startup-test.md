# 0667 — Test release deadlines without assuming child startup speed

Status: COMPLETE.
Landed: c3e7ed487737cce79d19a17c4e812c1a3456b879

Milestone: 5

## Outcome

No release test assumes that a child process starts, or that planning finishes, inside a short deadline it arms before that point. Three tests share this assumption: the mutation version check, the partial native outcomes case and the owned-child case. Each one accepts every valid outcome of its deadline, including expiry before the child starts, and still asserts what it exists to prove: one bounded version check and no plan; real partial results kept when natives have run; owned children cleaned up.

## Why

The batch gate failed twice on this class. First the version check's two-second budget expired before fake Cargo wrote its call log (j30742). Then the partial native outcomes case armed three seconds before planning and expired in the version step, before any partial outcome could exist (j30773). The owned-child test arms five seconds before readiness and has the same flaw. In each, production stopped correctly; the test assumed startup had finished.

## Proof

- Proof: retain both failed batch witnesses. For each of the three tests, cover expiry before the child starts and the case the test exists for. The version check records at most the one version call, reports the refusal and makes no plan. The partial outcomes case keeps its real partial-result assertions on a path where natives have run. The owned-child case keeps its cleanup assertions. Run the touched test files whole, get a fresh review, then one corrected batch gate.

## Starts from

- Starts from: `release/test_mutations.py::test_version_check_obeys_the_shared_deadline`, the partial native outcomes and owned-child deadline tests, and `release/mutations.py::tool_version`/`execute`. Second failure: candidate abbbfd9d8, raw j30773, on Yellow. The current-main test bytes match the failed candidate. Raw proof is on Yellow under `data/biomcp-redux-evidence/batch-0663-0665/11650a9aa5c72860335e1710e97ec68fa219e41f/failed-j30742/`.

## Keeps

- Keeps: production deadline, child cancellation, mutation scope and timing bounds. Do not widen the budget, skip or suppress deadline coverage, change production deadlines, use a fake clock over real I/O, or require the machine to schedule a child before an arbitrary short interval.

## Defers

- Defers: nothing.

## Changes

- Changes: the release test files holding these three tests; deadline assertions and the bounded before-start fixtures. Test code only.
