# 0667 — Test the mutation deadline without assuming child startup speed

Status: OPEN.

Milestone: 5

## Outcome

The mutation version-check regression accepts both valid outcomes of its shared deadline: expiry before Cargo starts, or expiry while its one version command hangs. It still rejects a fresh budget, an unbounded version check or a produced plan.

## Why

The batch gate failed because its two-second test budget expired before fake Cargo wrote its call log; the production command correctly stopped, but the test assumed that startup had completed.

## Proof

- Proof: retain the failed batch witness, cover the expired-before-start path and the existing hanging-version case through the mutation command, and run the existing mutation test file before the corrected batch gate. Any recorded call must be exactly the one version check, the refusal must be reported, and no plan may exist.

## Starts from

- Starts from: `release/test_mutations.py::test_version_check_obeys_the_shared_deadline` and `release/mutations.py::tool_version`/`execute`. The current-main test bytes match the failed candidate. Raw proof is on Yellow under `data/biomcp-redux-evidence/batch-0663-0665/11650a9aa5c72860335e1710e97ec68fa219e41f/failed-j30742/`.

## Keeps

- Keeps: production deadline, child cancellation, mutation scope and timing bounds. Do not widen the budget, skip deadline coverage, change production code or require the machine to schedule the fake child before an arbitrary short interval.

## Defers

- Defers: nothing.

## Changes

- Changes: `release/test_mutations.py`; deadline assertions and the bounded before-start fixture.
